"""The artifact uploaded to the public release workflow contains only reviewed files."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from build_linux_client_candidate import build
from client_install import InstallError, export, install, launch, metadata, target
from linux_client_asset_bundle import pack


@unittest.skipUnless(target() == 'linux-x86_64', 'the public candidate is Linux AMD64')
class ApprovedCandidate(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.repo = Path(folder.name)
        self.git('init', '-q')
        self.asset = self.repo / 'content' / 'approved.dat'
        self.asset.parent.mkdir()
        self.asset.write_bytes(b'verified public fixture')
        self.unapproved = self.asset.with_name('unapproved.dat')
        self.unapproved.write_bytes(b'excluded by the approval list')
        self.approval = self.repo / 'release' / 'linux-client-files.json'
        self.approval.parent.mkdir()
        self.output = self.repo / 'export'

    def git(self, *args):
        subprocess.run(['git', '-C', self.repo, *args], check=True)

    def approve(self, path):
        data = path.read_bytes()
        self.approval.write_text(json.dumps({'files': [{
            'source': path.relative_to(self.repo).as_posix(),
            'path': 'assets/approved.dat', 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(),
        }]}))
        self.git('add', str(self.approval))

    def build(self, **options):
        return build(self.repo, Path('/usr/bin/true'), self.approval, self.output, '0.1.0.12', **options)

    def approve_bundle(self, bundle):
        data = self.approval.read_text() if self.approval.exists() else '{"files":[]}'
        payload = bundle.read_bytes()
        approval = json.loads(data)
        approval['bundle'] = {'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()}
        self.approval.write_text(json.dumps(approval))
        self.git('add', str(self.approval))

    def test_only_reviewed_bytes_reach_candidate(self):
        self.git('add', 'content')
        self.approve(self.asset)
        out = self.build()
        self.assertEqual((out / 'payload/assets/approved.dat').read_bytes(), self.asset.read_bytes())
        manifest, _ = metadata(out)
        self.assertEqual({row['path'] for row in manifest['files']}, {'assets/approved.dat', 'theia-client'})

    def test_release_budget_measures_only_the_stripped_distribution_copy(self):
        source = self.repo / 'client.c'
        source.write_text('int main(void) { return 0; }\n')
        client = self.repo / 'client'
        subprocess.run(['cc', '-g', '-o', client, source], check=True)
        original = client.read_bytes()
        self.git('add', 'content')
        self.approve(self.asset)

        with patch('build_linux_client_candidate.MAX_RELEASE_BYTES', len(original) - 1):
            release = build(self.repo, client, self.approval, self.output, '0.1.0.12')

        executable = release / 'payload/theia-client'
        manifest, _ = metadata(release)
        entry = next(row for row in manifest['files'] if row['path'] == 'theia-client')
        self.assertEqual(client.read_bytes(), original)
        self.assertLess(entry['bytes'], len(original))
        self.assertEqual(entry['sha256'], hashlib.sha256(executable.read_bytes()).hexdigest())
        sections = subprocess.check_output(['readelf', '-SW', executable], text=True)
        self.assertNotIn('.debug_info', sections)
        self.assertNotIn('.symtab', sections)
        root = self.repo / 'installed'
        install(release, root)
        self.assertEqual(launch(root, []).wait(timeout=5), 0)

    def test_untracked_or_changed_files_are_refused(self):
        self.approve(self.asset)
        with self.assertRaisesRegex(InstallError, 'was not approved'):
            self.build()
        self.git('add', str(self.asset))
        self.asset.write_bytes(b'Verified public fixture')
        with self.assertRaisesRegex(InstallError, 'differs from the reviewed bytes'):
            self.build()
        self.assertFalse(self.output.exists())

    def test_pin_a_reproducible_bundle_beside_tracked_files(self):
        source = self.repo / 'source'
        source.mkdir()
        (source / 'theia-client').write_bytes(b'original binary')
        (source / 'terrain.height').write_bytes(b'reviewed ground')
        (source / 'navigation.json').write_bytes(b'reviewed navigation')
        (source / 'unreviewed.fbx').write_bytes(b'private source')
        original = export(source, self.repo / 'original', '0.1.0.11')
        one, two = self.repo / 'one.tar.gz', self.repo / 'two.tar.gz'
        paths = ['terrain.height', 'navigation.json']
        self.assertEqual(pack(original, paths, one), pack(original, reversed(paths), two))
        self.assertEqual(one.read_bytes(), two.read_bytes())

        self.git('add', 'content')
        self.approve(self.asset)
        self.approve_bundle(one)
        with self.assertRaisesRegex(InstallError, 'Supply the Linux asset bundle'):
            self.build()
        one.write_bytes(one.read_bytes()[:-1] + b'x')
        with self.assertRaisesRegex(InstallError, 'differs from the approved bytes'):
            self.build(asset_bundle=one)
        release = self.build(asset_bundle=two)
        manifest, _ = metadata(release)
        self.assertEqual({row['path'] for row in manifest['files']}, {
            'theia-client', 'terrain.height', 'navigation.json', 'assets/approved.dat',
        })
        self.assertEqual((release / 'payload/terrain.height').read_bytes(), b'reviewed ground')

    def test_bundle_members_cannot_escape_or_override_the_payload(self):
        for paths in (('../outside',), ('theia-client',), ('duplicate', 'duplicate')):
            with self.subTest(paths=paths):
                bundle = self.repo / 'malformed.tar.gz'
                with tarfile.open(bundle, 'w:gz') as archive:
                    for path in paths:
                        entry = tarfile.TarInfo(path)
                        entry.size = 1
                        archive.addfile(entry, io.BytesIO(b'x'))
                self.approve_bundle(bundle)
                with self.assertRaises(InstallError):
                    self.build(asset_bundle=bundle)
                self.assertFalse(self.output.exists())
                self.assertFalse((self.repo / 'outside').exists())
