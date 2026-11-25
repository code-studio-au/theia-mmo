"""Exercise signed client updates over a real, isolated HTTPS connection."""
from base64 import b64decode
from contextlib import contextmanager
import hashlib
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from random import Random
import ssl
import subprocess
import tempfile
import threading
import unittest
from unittest.mock import patch

from client_install import InstallError, active, export, installed, rollback
from stage_hosted_release import stage
from retain_pages_release import retain_previous
from hosted_release import HostedLibrary, NoRedirect, ZSTD, verified_payload
from urllib.error import URLError
from urllib.request import HTTPSHandler, build_opener


@unittest.skipUnless(Path('/usr/bin/dpkg-deb').is_file(), 'Linux packaging tools required')
class HostedDelivery(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / 'source'
        self.source.mkdir()
        (self.source / 'theia-client').write_bytes(b'qualified development client')
        (self.source / 'settings.json').write_bytes(b'{}')
        self.ui = self.source / 'ui/menu.bin'
        self.terrain = self.source / 'terrain/millhaven/land.bin'
        self.other_terrain = self.source / 'terrain/stoneford/land.bin'
        for path, contents in [(self.ui, b'interface assets'), (self.terrain, b'millhaven terrain'),
                               (self.other_terrain, b'stoneford terrain')]:
            path.parent.mkdir(parents=True)
            path.write_bytes(contents)
        self.key = self.root / 'signing.pem'
        self.public = self.root / 'public.der'
        self.tls_key = self.root / 'https-key.pem'
        self.cert = self.root / 'https.pem'
        self.openssl('genpkey', '-algorithm', 'ED25519', '-out', self.key)
        self.openssl('pkey', '-in', self.key, '-pubout', '-outform', 'DER', '-out', self.public)
        self.openssl('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
                     '-subj', '/CN=localhost', '-addext', 'subjectAltName=IP:127.0.0.1',
                     '-keyout', self.tls_key, '-out', self.cert)
        self.game = self.root / 'game'

    @staticmethod
    def openssl(*args):
        subprocess.run(['/usr/bin/openssl', *args], check=True, capture_output=True)

    def publish(self, version, *, bundle_initial_install=False):
        release = export(self.source, self.root / version, version)
        return stage(release, self.root / f'pages-{version}', '0.1.5', self.key,
                     bundle_initial_install=bundle_initial_install)

    def bundle_entries(self, tree, version):
        sidecar = tree / 'downloads' / 'bundles' / f'{version}.json'
        return verified_payload(json.loads(sidecar.read_bytes()), (self.public,))['entries']

    @contextmanager
    def serve(self, tree, *, faults=None):
        requests = []
        faults = faults or {}

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                request_range = self.headers.get('Range')
                requests.append((self.path, request_range))
                path = tree / self.path.lstrip('/')
                if not path.is_file():
                    self.send_error(404)
                    return
                pending = faults.get(self.path)
                error, limit = pending.pop(0) if pending else (None, None)
                if error is not None:
                    self.send_error(error)
                    return
                data = path.read_bytes()
                if request_range:
                    offset = int(request_range.removeprefix('bytes=').removesuffix('-'))
                    self.send_response(206)
                    self.send_header('Content-Range', f'bytes {offset}-{len(data) - 1}/{len(data)}')
                    data = data[offset:]
                else:
                    self.send_response(200)
                self.send_header('Content-Length', str(len(data)))
                self.end_headers()
                self.wfile.write(data[:limit])

        with ThreadingHTTPServer(('127.0.0.1', 0), Handler) as server:
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.load_cert_chain(self.cert, self.tls_key)
            server.socket = tls.wrap_socket(server.socket, server_side=True)
            worker = threading.Thread(target=server.serve_forever)
            worker.start()
            client_tls = ssl.create_default_context(cafile=str(self.cert))
            library = HostedLibrary(f'https://127.0.0.1:{server.server_port}/downloads/', (self.public,),
                                    opener=build_opener(NoRedirect(), HTTPSHandler(context=client_tls)))
            try:
                yield library, requests
            finally:
                server.shutdown()
                worker.join(timeout=5)

    def install(self, library, current=None):
        offer, version, _ = library.newest(current, '0.1.5', install_root=self.game)
        return offer.install(self.game, None, None, offered_version=version)

    def test_retries_connection_timeout_when_opening_download(self):
        with self.serve(self.publish('0.1.0.11')) as (library, _):
            offer, version, _ = library.newest(None, '0.1.5')
            open_request = library.opener.open

            def timed_out_once(*args, **kwargs):
                if opener.call_count == 1:
                    raise URLError(TimeoutError('timed out'))
                return open_request(*args, **kwargs)

            with patch.object(library.opener, 'open', side_effect=timed_out_once) as opener:
                with patch('hosted_release.sleep') as backoff:
                    self.assertEqual(offer.install(self.game, None, None, offered_version=version), version)
                backoff.assert_called_once_with(1)

    def test_retries_unavailable_server_and_resumes_truncated_file(self):
        payload = (self.source / 'theia-client').read_bytes()
        blob = '/downloads/blobs/' + hashlib.sha256(payload).hexdigest()
        faults = {blob: [(503, None), (None, 5)]}
        with self.serve(self.publish('0.1.0.11'), faults=faults) as (library, requests):
            with patch('hosted_release.sleep'):
                self.assertEqual(self.install(library), '0.1.0.11')
        self.assertEqual([offset for path, offset in requests if path == blob], [None, None, 'bytes=5-'])
        directory, _, _ = installed(self.game, '0.1.0.11')
        self.assertEqual((directory / 'payload/theia-client').read_bytes(), payload)

    def test_retry_limit_keeps_partial_file_for_next_install(self):
        payload = (self.source / 'theia-client').read_bytes()
        digest = hashlib.sha256(payload).hexdigest()
        blob = '/downloads/blobs/' + digest
        faults = {blob: [(None, 5), (503, None), (503, None)]}
        with self.serve(self.publish('0.1.0.11'), faults=faults) as (library, requests):
            with patch('hosted_release.sleep'):
                with self.assertRaisesRegex(InstallError, 'Choose Install Game or Update Game to resume'):
                    self.install(library)
            self.assertEqual([offset for path, offset in requests if path == blob], [None, 'bytes=5-', 'bytes=5-'])
            self.assertEqual((self.game / '.hosted-download' / digest).read_bytes(), payload[:5])
            self.assertIsNone(active(self.game)['current'])
            self.assertEqual(self.install(library), '0.1.0.11')
        self.assertEqual([offset for path, offset in requests if path == blob][-1], 'bytes=5-')

    def test_fresh_install_uses_bundles_and_update_only_downloads_changed_file(self):
        tree = self.publish('0.1.0.11', bundle_initial_install=True)
        bundles = self.bundle_entries(tree, '0.1.0.11')
        client, assets = [f'/downloads/blobs/{row["sha256"]}' for row in bundles]
        with self.serve(tree, faults={client: [(None, 10)]}) as (library, requests):
            offer, version, size = library.newest(None, '0.1.5', install_root=self.game)
            self.assertEqual(size, sum(row['bytes'] for row in bundles))
            with patch('hosted_release.sleep'):
                self.assertEqual(offer.install(self.game, None, None, offered_version=version), version)
            self.assertEqual([req for req in requests if '/blobs/' in req[0]],
                             [(client, None), (client, 'bytes=10-'), (assets, None)])
        directory, _, _ = installed(self.game, '0.1.0.11')
        self.assertEqual((directory / 'payload/ui/menu.bin').read_bytes(), self.ui.read_bytes())

        self.ui.write_bytes(b'updated interface assets')
        tree = self.publish('0.1.0.12', bundle_initial_install=True)
        with self.serve(tree) as (library, requests):
            _, _, size = library.newest('0.1.0.11', '0.1.5', install_root=self.game)
            self.assertEqual(size, self.ui.stat().st_size)
            self.assertEqual(self.install(library, '0.1.0.11'), '0.1.0.12')
            digest = hashlib.sha256(self.ui.read_bytes()).hexdigest()
            self.assertEqual([req for req in requests if '/blobs/' in req[0] or '/bundles/' in req[0]],
                             [(f'/downloads/blobs/{digest}', None)])

    def test_corrupt_bundle_falls_back_to_verified_individual_files(self):
        tree = self.publish('0.1.0.11', bundle_initial_install=True)
        _, assets = self.bundle_entries(tree, '0.1.0.11')
        asset_blob = tree / 'downloads' / 'blobs' / assets['sha256']
        asset_blob.write_bytes(b'x' * assets['bytes'])
        with self.serve(tree) as (library, requests):
            self.assertEqual(self.install(library), '0.1.0.11')
            for source in (self.source / 'theia-client', self.ui):
                digest = hashlib.sha256(source.read_bytes()).hexdigest()
                self.assertIn((f'/downloads/blobs/{digest}', None), requests)
        directory, _, _ = installed(self.game, '0.1.0.11')
        self.assertEqual((directory / 'payload/ui/menu.bin').read_bytes(), self.ui.read_bytes())

    def test_install_resume_update_and_rollback(self):
        payload = (self.source / 'theia-client').read_bytes()
        digest = hashlib.sha256(payload).hexdigest()
        cache = self.game / '.hosted-download'
        cache.mkdir(parents=True)
        (cache / digest).write_bytes(payload[:5])
        with self.serve(self.publish('0.1.0.11')) as (library, requests):
            self.assertEqual(self.install(library), '0.1.0.11')
            self.assertIn((f'/downloads/blobs/{digest}', 'bytes=5-'), requests)
            self.assertIsNone(library.newest('0.1.0.11', '0.1.5'))
        client = b'updated development client'
        (self.source / 'theia-client').write_bytes(client)
        executable_update = self.publish('0.1.0.12')
        with self.serve(executable_update) as (library, requests):
            _, _, download = library.newest('0.1.0.11', '0.1.5', install_root=self.game)
            self.assertEqual(download, len(client))
            self.assertEqual(self.install(library, '0.1.0.11'), '0.1.0.12')
            self.assertEqual([path for path, _ in requests if '/blobs/' in path],
                             ['/downloads/blobs/' + hashlib.sha256(client).hexdigest()])
        terrain = b'updated millhaven terrain'
        self.terrain.write_bytes(terrain)
        terrain_update = self.publish('0.1.0.13')
        with self.serve(terrain_update) as (library, requests):
            _, _, download = library.newest('0.1.0.12', '0.1.5', install_root=self.game)
            self.assertEqual(download, len(terrain))
            self.assertEqual(self.install(library, '0.1.0.12'), '0.1.0.13')
            self.assertEqual([path for path, _ in requests if '/blobs/' in path],
                             ['/downloads/blobs/' + hashlib.sha256(terrain).hexdigest()])
        def versions(tree):
            from base64 import b64decode
            envelope = json.loads((tree / 'downloads/development.json').read_bytes())
            packages = json.loads(b64decode(envelope['payload']))['packages']
            return {row['id']: row['version'] for row in packages}
        previous, current = versions(executable_update), versions(terrain_update)
        self.assertEqual(set(current), {'client', 'gameplay', 'terrain/millhaven', 'terrain/stoneford', 'ui'})
        self.assertEqual({key for key in current if current[key] != previous[key]}, {'terrain/millhaven'})
        self.assertEqual(rollback(self.game), '0.1.0.12')
        directory, _, _ = installed(self.game, '0.1.0.12')
        self.assertEqual((directory / 'payload/terrain/millhaven/land.bin').read_bytes(), b'millhaven terrain')

    @unittest.skipUnless(ZSTD.is_file(), 'zstd is required for binary patch qualification')
    def test_binary_patch_resume_and_corrupt_patch_full_file_fallback(self):
        executable = self.source / 'theia-client'
        initial = Random(1729).randbytes(512 * 1024)
        executable.write_bytes(initial)
        old = export(self.source, self.root / 'base', '0.1.0.11')
        base_pages = stage(old, self.root / 'base-pages', '0.1.5', self.key)
        with self.serve(base_pages) as (library, _):
            self.install(library)

        target = initial[:100_000] + b'one changed byte range' + initial[100_022:]
        executable.write_bytes(target)
        new = export(self.source, self.root / 'target', '0.1.0.12')
        tree = stage(new, self.root / 'target-pages', '0.1.5', self.key, delta_base=old)
        sidecar = json.loads((tree / 'downloads/deltas/0.1.0.12.json').read_bytes())
        patch, = json.loads(b64decode(sidecar['payload']))['entries']
        patch_url = '/downloads/blobs/' + patch['sha256']
        full_url = '/downloads/blobs/' + hashlib.sha256(target).hexdigest()
        cache = self.game / '.hosted-download'
        (cache / patch['sha256']).write_bytes((tree / patch_url.lstrip('/')).read_bytes()[:32])

        with self.serve(tree) as (library, requests):
            offer, version, size = library.newest('0.1.0.11', '0.1.8', install_root=self.game)
            self.assertLess(size, len(target) // 100)
            self.assertEqual(size, patch['bytes'])
            self.assertEqual(offer.install(self.game, None, None, offered_version=version), '0.1.0.12')
            self.assertIn((patch_url, 'bytes=32-'), requests)
            self.assertFalse(any(path == full_url for path, _ in requests))
        installed_dir, _, _ = installed(self.game, '0.1.0.12')
        self.assertEqual((installed_dir / 'payload/theia-client').read_bytes(), target)

        for problem in ('corrupt patch', 'invalid signature'):
            with self.subTest(problem=problem):
                self.game = self.root / problem
                with self.serve(base_pages) as (library, _):
                    self.install(library)
                (tree / patch_url.lstrip('/')).write_bytes(b'x' * patch['bytes'])
                if problem == 'invalid signature':
                    (tree / 'downloads/deltas/0.1.0.12.json').write_text(
                        json.dumps({**sidecar, 'signature': 'bad!'}))
                with self.serve(tree) as (library, requests):
                    self.assertEqual(self.install(library, '0.1.0.11'), '0.1.0.12')
                    self.assertIn((full_url, None), requests)
                    self.assertEqual((patch_url, None) in requests, problem == 'corrupt patch')
                directory, _, _ = installed(self.game, '0.1.0.12')
                self.assertEqual((directory / 'payload/theia-client').read_bytes(), target)
                self.assertEqual(rollback(self.game), '0.1.0.11')

    @unittest.skipUnless(ZSTD.is_file(), 'zstd is needed to verify retention of old patch blobs')
    def test_second_pages_release_keeps_previous_installer_files_delta_and_bundles(self):
        entry = self.source / 'theia-client'
        initial = Random(19).randbytes(300 * 1024)
        entry.write_bytes(initial)
        base = export(self.source, self.root / 'pre-release', '0.1.0.10')
        previous_payload = b'old' + initial[3:]
        entry.write_bytes(previous_payload)
        previous = export(self.source, self.root / 'first-export', '0.1.0.11')
        with self.serve(self.root / 'empty') as (library, _):
            first = stage(previous, self.root / 'first-pages', '0.1.8', self.key, delta_base=base,
                          live_mode='first', live_url=library.url, live_opener=library.opener,
                          release_notes={'launcher': 'First launcher', 'client': 'First game client'},
                          bundle_initial_install=True)

        entry.write_bytes(b'new' + initial[3:])
        current = export(self.source, self.root / 'second-export', '0.1.0.12')
        with self.serve(first) as (library, _):
            with self.assertRaisesRegex(InstallError, 'expected previous release'):
                retain_previous(library, self.root / 'unused-downloads', {'version': '0.1.0.12'},
                                '0.1.9', first_release=False, expected_previous='0.1.0.10')
            second = stage(current, self.root / 'second-pages', '0.1.9', self.key,
                           live_mode='update', live_url=library.url, live_opener=library.opener,
                           expected_previous='0.1.0.11',
                           release_notes={'launcher': 'Launcher update', 'client': 'Client update'},
                           bundle_initial_install=True)

        old_downloads, new_downloads = first / 'downloads', second / 'downloads'
        stable = 'theia-launcher_amd64.deb'
        self.assertEqual((new_downloads / stable).read_bytes(),
                         (new_downloads / 'theia-launcher_0.1.9_amd64.deb').read_bytes())
        self.assertEqual((new_downloads / 'theia-launcher_0.1.8_amd64.deb').read_bytes(),
                         (old_downloads / stable).read_bytes())
        sidecar = 'deltas/0.1.0.11.json'
        self.assertEqual((new_downloads / sidecar).read_bytes(), (old_downloads / sidecar).read_bytes())
        delta, = json.loads(b64decode(json.loads((new_downloads / sidecar).read_bytes())['payload']))['entries']
        self.assertEqual((new_downloads / 'blobs' / delta['sha256']).read_bytes(),
                         (old_downloads / 'blobs' / delta['sha256']).read_bytes())
        def verify_retained_bundles(before, after, version):
            index = f'downloads/bundles/{version}.json'
            self.assertEqual((after / index).read_bytes(), (before / index).read_bytes())
            for bundle in self.bundle_entries(before, version):
                blob = f'downloads/blobs/{bundle["sha256"]}'
                self.assertEqual((after / blob).read_bytes(), (before / blob).read_bytes())

        verify_retained_bundles(first, second, '0.1.0.11')
        old_blob = '/downloads/blobs/' + hashlib.sha256(previous_payload).hexdigest()
        with self.serve(second) as (library, requests):
            with library.open(old_blob.removeprefix('/downloads/'), offset=17) as response:
                self.assertEqual(response.read(), previous_payload[17:])
            self.assertIn((old_blob, 'bytes=17-'), requests)
            _, version, _ = library.newest(None, '0.1.9')
            self.assertEqual(version, '0.1.0.12')
            launcher_only = stage(current, self.root / 'launcher-only-pages', '0.1.10', self.key,
                                  live_mode='update', live_url=library.url, live_opener=library.opener,
                                  expected_previous='0.1.0.12',
                                  release_notes={'launcher': 'Launcher-only update'},
                                  bundle_initial_install=True)
        verify_retained_bundles(second, launcher_only, '0.1.0.12')
        newest_downloads = launcher_only / 'downloads'
        self.assertEqual((newest_downloads / stable).read_bytes(),
                         (newest_downloads / 'theia-launcher_0.1.10_amd64.deb').read_bytes())
        self.assertEqual((newest_downloads / 'theia-launcher_0.1.9_amd64.deb').read_bytes(),
                         (new_downloads / stable).read_bytes())
        entries = verified_payload(json.loads((newest_downloads / 'release-news.json').read_bytes()),
                                   (self.public,))['entries']
        self.assertEqual([(row['product'], row['version']) for row in entries], [
            ('launcher', '0.1.8'), ('client', '0.1.0.11'), ('launcher', '0.1.9'),
            ('client', '0.1.0.12'), ('launcher', '0.1.10')])
        self.assertEqual(entries[-1]['notes'], ['Launcher-only update'])

    def test_rejects_corrupt_download_and_preserves_current_version(self):
        with self.serve(self.publish('0.1.0.11')) as (library, _):
            self.install(library)
        payload = b'new binary'
        (self.source / 'theia-client').write_bytes(payload)
        tree = self.publish('0.1.0.12')
        (tree / 'downloads/blobs' / hashlib.sha256(payload).hexdigest()).write_bytes(b'x' * len(payload))
        with self.serve(tree) as (library, _):
            with self.assertRaisesRegex(InstallError, 'Download verification failed'):
                self.install(library, '0.1.0.11')
        self.assertEqual(active(self.game)['current'], '0.1.0.11')
        self.assertFalse((self.game / 'versions/0.1.0.12').exists())

    def test_unsigned_metadata_or_old_launcher_does_not_offer_a_download(self):
        tree = self.publish('0.1.0.11')
        with self.serve(tree) as (library, requests):
            with self.assertRaisesRegex(InstallError, 'latest launcher'):
                library.newest(None, '0.1.4')
            self.assertEqual(requests, [('/downloads/development.json', None)])
            manifest = tree / 'downloads/development.json'
            manifest.write_bytes(manifest.read_bytes().replace(b'"signature": "', b'"signature": "aaaa'))
            with self.assertRaisesRegex(InstallError, 'signature|encoding'):
                library.newest(None, '0.1.5')


if __name__ == '__main__':
    unittest.main()
