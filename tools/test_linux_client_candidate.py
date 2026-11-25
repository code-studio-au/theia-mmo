"""The artifact uploaded to the public release workflow contains only reviewed files."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from build_linux_client_candidate import build
from client_install import InstallError, install, launch, metadata, target


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

    def build(self):
        return build(self.repo, Path('/usr/bin/true'), self.approval, self.output, '0.1.0.12')

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
