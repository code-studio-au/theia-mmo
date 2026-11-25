"""Signed Development releases served as immutable files over HTTPS."""
import base64
from dataclasses import dataclass
import hashlib
import http.client
import json
import os
import re
from pathlib import Path
import subprocess
import tempfile
from threading import Timer
from time import sleep
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener
from zipfile import BadZipFile, ZIP_DEFLATED, ZIP_STORED, ZipFile
from zlib import error as CompressionError

from client_install import (
    CHUNK_BYTES, MAX_MANIFEST_BYTES, InstallError, active, copy_file, decode_json,
    install_release, installed, regular, validate_metadata, version_key,
)

# A separately deployed Pages project keeps packages live across website builds.
DEFAULT_URL = 'https://code-studio-au.github.io/theia-mmo/downloads/'
CHANNEL_FILE = 'development.json'
CHANNEL = 'Development'
LAUNCHER_FEED = 'launcher.json'
LAUNCHER_FORMAT = 'theia-launcher-deb-amd64-v1'
MAX_LAUNCHER_FEED = 8192
MAX_INSTALLER = 64 * 1024**2
MAX_ENVELOPE = 6 * 1024**2
MAX_RELEASE_BYTES = 950_000_000
DOWNLOAD_ATTEMPTS = 3
RETRYABLE_HTTP_STATUS = {408, 500, 502, 503, 504}
BUNDLE_FORMAT = 'zip-v1'
MAX_BUNDLE_METADATA = 8192
DELTA_MIN_BYTES = 256 * 1024
DELTA_MAX_BYTES = 256 * 1024**2
DELTA_LIMIT = 256
ZSTD = Path('/usr/bin/zstd')
ED25519_PUBLIC_DER_PREFIX = bytes.fromhex('302a300506032b6570032100')


def package_id(path):
    parts = path.split('/')
    if len(parts) == 1 and parts[0] == 'theia-client':
        return 'client'
    if parts[0] in ('assets', 'content') and len(parts) > 1:
        parts = parts[1:]
    if len(parts) > 2 and parts[0] in ('terrain', 'zones'):
        return f'terrain/{parts[1]}'
    if len(parts) > 1 and parts[0] in ('ui', 'characters', 'equipment', 'animations', 'objects', 'audio'):
        return parts[0]
    return 'gameplay'


def release_packages(files):
    """Give each package a stable ID and a version derived from its file entries."""
    groups = {}
    for row in files:
        groups.setdefault(package_id(row['path']), []).append(row)
    packages = []
    for name, rows in sorted(groups.items()):
        canonical = json.dumps(sorted(rows, key=lambda row: row['path']),
                               sort_keys=True, separators=(',', ':')).encode('ascii')
        packages.append({'id': name, 'version': hashlib.sha256(canonical).hexdigest(),
                         'bytes': sum(row['bytes'] for row in rows)})
    return packages


def content_key(row):
    return row['sha256'], row['bytes']


def validate_launcher_metadata(data):
    if (not isinstance(data, dict) or set(data) != {'format', 'version', 'bytes', 'sha256'}
            or data['format'] != LAUNCHER_FORMAT
            or type(data['bytes']) is not int or not 0 < data['bytes'] <= MAX_INSTALLER
            or not isinstance(data['sha256'], str) or not re.fullmatch('[a-f0-9]{64}', data['sha256'])):
        raise InstallError('Invalid launcher update metadata')
    version_key(data['version'])
    return data


def bundle_groups(manifest):
    groups = {}
    for row in manifest['files']:
        group = 'client' if row['path'] == 'theia-client' else 'assets'
        groups.setdefault(group, {})[row['path']] = row
    return groups


def validate_bundles(data, manifest):
    groups = bundle_groups(manifest)
    if (not isinstance(data, dict) or set(data) != {'format', 'version', 'entries'}
            or data['format'] != BUNDLE_FORMAT or data['version'] != manifest['version']
            or not isinstance(data['entries'], list) or len(data['entries']) != len(groups)):
        raise InstallError('Invalid install bundle metadata')
    bundles = {}
    for entry in data['entries']:
        if (not isinstance(entry, dict) or set(entry) != {'id', 'bytes', 'sha256'}
                or not isinstance(entry['id'], str) or entry['id'] not in groups or entry['id'] in bundles
                or type(entry['bytes']) is not int or not 0 < entry['bytes'] <= MAX_RELEASE_BYTES
                or not isinstance(entry['sha256'], str) or not re.fullmatch('[a-f0-9]{64}', entry['sha256'])):
            raise InstallError('Invalid install bundle entry')
        bundles[entry['id']] = entry
    if sum(entry['bytes'] for entry in bundles.values()) > MAX_RELEASE_BYTES:
        raise InstallError('Install bundles exceed the download size limit')
    return bundles


def unpack_bundle(path, rows, destination, check_cancelled):
    """Write only exact, signed file paths and validate bytes before activation."""
    try:
        with ZipFile(path) as archive:
            entries = archive.infolist()
            if len(entries) != len(rows) or {entry.filename for entry in entries} != rows.keys():
                raise InstallError('Install bundle does not match its release file list')
            for entry in entries:
                row = rows[entry.filename]
                if (entry.file_size != row['bytes'] or entry.flag_bits & 1
                        or entry.compress_type not in (ZIP_DEFLATED, ZIP_STORED)):
                    raise InstallError('Invalid file in install bundle')
                target = destination / row['path']
                target.parent.mkdir(parents=True, exist_ok=True)
                digest = hashlib.sha256()
                size = 0
                with archive.open(entry) as source, target.open('xb') as output:
                    while chunk := source.read(min(CHUNK_BYTES, row['bytes'] - size + 1)):
                        size += len(chunk)
                        if size > row['bytes']:
                            raise InstallError('Install bundle file exceeds its signed size')
                        output.write(chunk)
                        digest.update(chunk)
                        check_cancelled()
                    if (size, digest.hexdigest()) != (row['bytes'], row['sha256']):
                        raise InstallError('Install bundle file failed verification')
                    output.flush()
                    os.fsync(output.fileno())
    except (BadZipFile, CompressionError, EOFError, NotImplementedError) as error:
        raise InstallError('Could not read install bundle') from error


def delta_eligible(old, new):
    return bool(old and old['sha256'] != new['sha256']
                and 1 <= old['bytes'] <= DELTA_MAX_BYTES
                and DELTA_MIN_BYTES <= new['bytes'] <= DELTA_MAX_BYTES)


def delta_candidates(previous_files, deltas):
    old = {row['path']: row for row in previous_files}
    return {name: patch for name, patch in deltas.items()
            if name in old and content_key(old[name]) == (patch['base_sha256'], patch['base_bytes'])}


def validate_deltas(data, manifest):
    if (not isinstance(data, dict) or set(data) != {'format', 'version', 'entries'}
            or data['format'] != 'zstd-patch-v1' or data['version'] != manifest['version']
            or not isinstance(data['entries'], list) or len(data['entries']) > DELTA_LIMIT):
        raise InstallError('Invalid binary patch metadata')
    targets = {row['path']: row for row in manifest['files']}
    patches = {}
    for entry in data['entries']:
        if not isinstance(entry, dict) or set(entry) != {
                'path', 'base_sha256', 'base_bytes', 'target_sha256', 'sha256', 'bytes'}:
            raise InstallError('Invalid binary patch entry')
        path = entry['path']
        target = targets.get(path) if isinstance(path, str) else None
        if (target is None or path in patches or entry['target_sha256'] != target['sha256']
                or not DELTA_MIN_BYTES <= target['bytes'] <= DELTA_MAX_BYTES
                or entry['base_sha256'] == target['sha256']
                or any(not isinstance(entry[key], str) or not re.fullmatch('[a-f0-9]{64}', entry[key])
                       for key in ('sha256', 'base_sha256'))
                or type(entry['base_bytes']) is not int or not 1 <= entry['base_bytes'] <= DELTA_MAX_BYTES
                or type(entry['bytes']) is not int or not 0 < entry['bytes'] * 5 <= target['bytes'] * 4):
            raise InstallError('Binary patch does not match the signed release')
        patches[path] = entry
    return patches


def apply_delta(base, patch, destination, row, advance):
    """Bound the decoded output by the signed target size and verify it before use."""
    digest = hashlib.sha256()
    size = 0
    with subprocess.Popen([ZSTD, '-dq', '-M512MB', f'--patch-from={base}', '-c', '--', patch],
                          stdout=subprocess.PIPE, stderr=subprocess.DEVNULL) as decoder:
        timeout = Timer(60, decoder.kill)
        timeout.start()
        try:
            with destination.open('xb') as output:
                while chunk := decoder.stdout.read(min(CHUNK_BYTES, row['bytes'] - size + 1)):
                    size += len(chunk)
                    if size > row['bytes']:
                        raise InstallError('Binary patch exceeds its signed target size')
                    output.write(chunk)
                    digest.update(chunk)
                    advance(size)
                if decoder.wait() or (size, digest.hexdigest()) != (row['bytes'], row['sha256']):
                    raise InstallError('Binary patch verification failed')
                output.flush()
                os.fsync(output.fileno())
        finally:
            timeout.cancel()
            if decoder.poll() is None:
                decoder.kill()
            timeout.join()


class DownloadInterrupted(InstallError):
    pass


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


@dataclass(frozen=True)
class HostedRelease:
    library: 'HostedLibrary'
    manifest: dict
    total: int
    deltas: dict
    bundles: dict

    def download_size(self, previous_files=()):
        if self.bundles and not previous_files:
            return sum(row['bytes'] for row in self.bundles.values())
        known = {content_key(row) for row in previous_files}
        patches = delta_candidates(previous_files, self.deltas)
        missing = set()
        for row in self.manifest['files']:
            if content_key(row) not in known:
                missing.add(content_key(patches.get(row['path'], row)))
        return sum(size for _, size in missing)

    def install(self, root, progress, cancelled, *, offered_version):
        root = root.resolve()
        return install_release(root, self.manifest, self.total,
                               lambda stage: self.populate(root, stage, progress, cancelled),
                               progress, cancelled, offered_version=offered_version)

    def populate(self, root, stage, progress, cancelled):
        cache = root / '.hosted-download'
        cache.mkdir(mode=0o700, exist_ok=True)
        if cache.is_symlink():
            raise InstallError('The download cache must be a local directory')
        digests = {row['sha256'] for row in self.manifest['files']}
        digests.update(patch['sha256'] for patch in self.deltas.values())
        digests.update(bundle['sha256'] for bundle in self.bundles.values())
        for entry in cache.iterdir():
            if entry.name not in digests:
                entry.unlink()

        current = active(root)['current']
        if current or not self.bundles or not self.try_bundles(cache, stage, progress, cancelled):
            self.populate_files(root, current, cache, stage, progress, cancelled)

        # Only verified bytes reach this point. Activation remains under the installer lock.
        for entry in cache.iterdir():
            entry.unlink()

    def try_bundles(self, cache, stage, progress, cancelled):
        groups = bundle_groups(self.manifest)
        complete = 0
        total = self.download_size()

        def check_cancelled():
            if cancelled and cancelled():
                raise InstallError('Update cancelled')

        def advance(count):
            check_cancelled()
            if progress:
                progress(complete + count, total)

        with tempfile.TemporaryDirectory(prefix='bundles-', dir=stage) as folder:
            payload = Path(folder) / 'payload'
            try:
                for name, row in self.bundles.items():
                    archive = cache / row['sha256']
                    self.library.download(row, archive, advance)
                    if copy_file(archive, None, row['bytes'], lambda _: check_cancelled()) != (row['bytes'], row['sha256']):
                        archive.unlink()
                        raise InstallError('Install bundle failed verification')
                    unpack_bundle(archive, groups[name], payload, check_cancelled)
                    complete += row['bytes']
            except DownloadInterrupted:
                raise
            except InstallError:
                check_cancelled()
                return False
            payload.rename(stage / 'payload')
        return True

    def populate_files(self, root, current, cache, stage, progress, cancelled):
        old_directory, old_files, patches = None, {}, {}
        if current:
            old_directory, manifest, _ = installed(root, current)
            old_files = {content_key(row): row['path'] for row in manifest['files']}
            patches = delta_candidates(manifest['files'], self.deltas)

        completed = 0
        for row in self.manifest['files']:
            name = row['path']
            destination = stage / 'payload' / name
            destination.parent.mkdir(parents=True, exist_ok=True)

            def advance(count):
                if cancelled and cancelled():
                    raise InstallError('Update cancelled')
                if progress:
                    progress(completed + count, self.total)

            advance(0)
            reused = False
            old_path = old_files.get(content_key(row))
            if old_path is not None:
                try:
                    reused = copy_file(old_directory / 'payload' / old_path, destination,
                                       row['bytes'], lambda _: None) == (row['bytes'], row['sha256'])
                except (OSError, InstallError):
                    pass
                if not reused:
                    destination.unlink(missing_ok=True)
            if not reused and name in patches and ZSTD.is_file():
                patch = patches[name]
                base = old_directory / 'payload' / name
                partial = cache / patch['sha256']
                try:
                    if copy_file(base, None, patch['base_bytes'], lambda _: advance(0)) != (
                            patch['base_bytes'], patch['base_sha256']):
                        raise InstallError('Installed base does not match binary patch')
                    self.library.download(patch, partial, advance)
                    if copy_file(partial, None, patch['bytes'], lambda _: advance(0)) != (patch['bytes'], patch['sha256']):
                        partial.unlink()
                        raise InstallError('Binary patch download failed verification')
                    apply_delta(base, partial, destination, row, advance)
                    reused = True
                except DownloadInterrupted:
                    raise
                except (OSError, InstallError):
                    destination.unlink(missing_ok=True)
                    if cancelled and cancelled():
                        raise InstallError('Update cancelled') from None
            if not reused:
                partial = cache / row['sha256']
                self.library.download(row, partial, advance)
                if copy_file(partial, destination, row['bytes'], lambda _: None) != (
                        row['bytes'], row['sha256']):
                    partial.unlink()
                    raise InstallError(f'Download verification failed: {name}')
            advance(row['bytes'])
            completed += row['bytes']


class HostedLibrary:
    def __init__(self, url, public_keys, *, opener=None):
        parsed = urlsplit(url)
        if (parsed.scheme != 'https' or not parsed.hostname or parsed.username is not None
                or parsed.query or parsed.fragment or not parsed.path.endswith('/')):
            raise InstallError('Hosted releases require an HTTPS directory')
        self.url = url
        self.public_keys = tuple(public_keys)
        self.opener = opener or build_opener(NoRedirect())

    def open(self, name, *, offset=0):
        headers = {'Accept-Encoding': 'identity', 'User-Agent': 'TheiaLauncher'}
        if offset:
            headers['Range'] = f'bytes={offset}-'
        request = Request(self.url + name, headers=headers)
        try:
            response = self.opener.open(request, timeout=20)
        except HTTPError as error:
            error.close()
            if error.code in RETRYABLE_HTTP_STATUS:
                raise DownloadInterrupted('The download server is temporarily unavailable') from error
            raise InstallError('Could not reach the Theia download server. Try again.') from error
        except (URLError, TimeoutError, ConnectionError, http.client.HTTPException) as error:
            reason = error.reason if isinstance(error, URLError) else error
            if isinstance(reason, (TimeoutError, ConnectionError, http.client.HTTPException)):
                raise DownloadInterrupted('The connection to the download server was interrupted') from error
            raise InstallError('Could not reach the Theia download server. Try again.') from error
        if response.geturl() != request.full_url or response.headers.get('Content-Encoding', 'identity') != 'identity':
            response.close()
            raise InstallError('The download server returned an unexpected response')
        return response

    def newest(self, current, launcher_version, *, install_root=None):
        try:
            with self.open(CHANNEL_FILE) as response:
                data = response.read(MAX_ENVELOPE + 1)
        except (http.client.HTTPException, TimeoutError) as error:
            raise InstallError('Could not read the Theia release list. Try again.') from error
        if len(data) > MAX_ENVELOPE:
            raise InstallError('Hosted release metadata exceeds its size limit')
        payload = verify_envelope(decode_json(data), self.public_keys)
        manifest, total = validate_metadata(payload['client'])
        if 'packages' in payload and payload['packages'] != release_packages(manifest['files']):
            raise InstallError('Hosted package metadata does not match its client files')
        if not 0 < total <= MAX_RELEASE_BYTES:
            raise InstallError('Hosted client exceeds the download size limit')
        if current and version_key(manifest['version']) <= version_key(current):
            return None
        if version_key(launcher_version) < version_key(payload['minimum_launcher']):
            raise InstallError('Download the latest launcher from the Theia Online website to update the game.')
        previous_files = ()
        deltas = {}
        if current and install_root is not None:
            _, previous, _ = installed(install_root, current)
            previous_files = previous['files']
            old_by_path = {row['path']: row for row in previous_files}
            if ZSTD.is_file() and any(delta_eligible(old_by_path.get(row['path']), row)
                                      for row in manifest['files']):
                deltas = self.deltas(manifest)
        bundles = self.bundles(manifest) if not current else {}
        release = HostedRelease(self, manifest, total, deltas, bundles)
        return release, manifest['version'], release.download_size(previous_files)

    def bundles(self, manifest):
        try:
            with self.open(f"bundles/{manifest['version']}.json") as response:
                data = response.read(MAX_BUNDLE_METADATA + 1)
            if len(data) > MAX_BUNDLE_METADATA:
                raise InstallError('Install bundle metadata exceeds its size limit')
            signed = verified_payload(decode_json(data), self.public_keys)
            return validate_bundles(signed, manifest)
        except (InstallError, OSError, http.client.HTTPException):
            return {}

    def launcher_metadata(self):
        try:
            with self.open(LAUNCHER_FEED) as response:
                data = response.read(MAX_LAUNCHER_FEED + 1)
        except InstallError as error:
            if isinstance(error.__cause__, HTTPError) and error.__cause__.code == 404:
                return None
            raise
        except (http.client.HTTPException, TimeoutError, ConnectionError) as error:
            raise InstallError('Could not read launcher updates. Try again.') from error
        if len(data) > MAX_LAUNCHER_FEED:
            raise InstallError('Launcher update metadata exceeds its size limit')
        return validate_launcher_metadata(verified_payload(decode_json(data), self.public_keys))

    def deltas(self, manifest):
        try:
            with self.open(f"deltas/{manifest['version']}.json") as response:
                data = response.read(MAX_ENVELOPE + 1)
            if len(data) > MAX_ENVELOPE:
                raise InstallError('Binary patch metadata exceeds its size limit')
            signed = verified_payload(decode_json(data), self.public_keys)
            return validate_deltas(signed, manifest)
        except (InstallError, OSError, http.client.HTTPException):
            # The signed release remains usable with full file downloads.
            return {}

    def download(self, row, partial, progress, *, name=None):
        source = name or 'blobs/' + row['sha256']
        for attempt in range(DOWNLOAD_ATTEMPTS):
            try:
                self._download_once(row, partial, progress, source)
                return
            except DownloadInterrupted as error:
                if attempt + 1 == DOWNLOAD_ATTEMPTS:
                    raise DownloadInterrupted('Download interrupted. Try again to resume.') from error
                sleep(2 ** attempt)

    def _download_once(self, row, partial, progress, source):
        size = row['bytes']
        offset = regular(partial).stat().st_size if partial.exists() or partial.is_symlink() else 0
        if offset > size:
            partial.unlink()
            offset = 0
        progress(offset)
        if size == 0:
            partial.touch()
            return
        if offset == size:
            return
        with self.open(source, offset=offset) as response:
            if response.status == 200:
                offset = 0
            elif (not offset or response.status != 206 or response.headers.get('Content-Range')
                  != f'bytes {offset}-{size - 1}/{size}'):
                raise InstallError('The download server returned an invalid byte range')
            mode = 'ab' if offset else 'wb'
            with partial.open(mode) as output:
                try:
                    while chunk := response.read(min(CHUNK_BYTES, size - offset + 1)):
                        if len(chunk) > size - offset:
                            raise InstallError('Downloaded file exceeds its declared size')
                        output.write(chunk)
                        offset += len(chunk)
                        progress(offset)
                except (http.client.HTTPException, TimeoutError, ConnectionError) as error:
                    raise DownloadInterrupted('The download was interrupted') from error
        if offset != size:
            raise DownloadInterrupted('The download was interrupted')


def verified_payload(envelope, public_keys):
    if not isinstance(envelope, dict) or set(envelope) != {'payload', 'signature'}:
        raise InstallError('Invalid hosted release envelope')
    try:
        payload = base64.b64decode(envelope['payload'], validate=True)
        signature = base64.b64decode(envelope['signature'], validate=True)
    except (TypeError, ValueError) as error:
        raise InstallError('Invalid hosted release encoding') from error
    if not 0 < len(payload) <= MAX_MANIFEST_BYTES or len(signature) != 64:
        raise InstallError('Invalid hosted release signature or metadata size')
    with tempfile.TemporaryDirectory(prefix='theia-signature-') as folder:
        stage = Path(folder)
        body, signed = stage / 'payload', stage / 'signature'
        body.write_bytes(payload)
        signed.write_bytes(signature)
        for key in public_keys:
            der = key.read_bytes()
            if len(der) != len(ED25519_PUBLIC_DER_PREFIX) + 32 or not der.startswith(ED25519_PUBLIC_DER_PREFIX):
                raise InstallError('Invalid installed release verification key')
            try:
                result = subprocess.run(
                    ['/usr/bin/openssl', 'pkeyutl', '-verify', '-rawin', '-pubin', '-keyform', 'DER',
                     '-inkey', str(key), '-in', str(body), '-sigfile', str(signed)],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10,
                    check=False,
                )
            except subprocess.TimeoutExpired as error:
                raise InstallError('Release signature verification timed out') from error
            if result.returncode == 0:
                break
        else:
            raise InstallError('Could not verify the Theia release signature')
    return decode_json(payload)


def verify_envelope(envelope, public_keys):
    data = verified_payload(envelope, public_keys)
    fields = {'channel', 'minimum_launcher', 'client'}
    if (not isinstance(data, dict) or set(data) not in (fields, fields | {'packages'})
            or data['channel'] != CHANNEL):
        raise InstallError('Unknown hosted release format or channel')
    version_key(data['minimum_launcher'])
    return data
