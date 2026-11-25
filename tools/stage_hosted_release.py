"""Prepare an approved Linux client export and launcher for GitHub Pages."""
import argparse
import base64
import hashlib
from pathlib import Path
import subprocess
import tempfile

from client_install import MANIFEST, InstallError, copy_file, metadata, version, write_json
from package_client_launcher import package
from hosted_release import CHANNEL, CHANNEL_FILE, MAX_RELEASE_BYTES, release_packages, verify_envelope


def stage(release, output, launcher_version, signing_key, retained=()):
    output = output.resolve()
    launcher_version = version(launcher_version)
    if output.exists():
        raise InstallError('Choose a new Pages artifact folder')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.pages-', dir=output.parent) as temporary:
        scratch = Path(temporary)
        tree = scratch / 'pages'
        downloads = tree / 'downloads'
        blobs = downloads / 'blobs'
        blobs.mkdir(parents=True)
        used = 0
        manifest = None
        for candidate in (*retained, release):
            candidate = candidate.resolve(strict=True)
            manifest, total = metadata(candidate)
            if manifest['target'] != 'linux-x86_64' or total > MAX_RELEASE_BYTES:
                raise InstallError('Expected a Linux client export within the Pages size budget')
            for row in manifest['files']:
                destination = blobs / row['sha256']
                new = not destination.exists()
                if new:
                    used += row['bytes']
                    if used > MAX_RELEASE_BYTES:
                        raise InstallError('Retained packages exceed the Pages size budget')
                if copy_file(candidate / 'payload' / row['path'], destination if new else None,
                             row['bytes'], lambda _: None) != (row['bytes'], row['sha256']):
                    raise InstallError(f'Client export failed verification: {row["path"]}')

        root_key = scratch / 'public.der'
        subprocess.run(['/usr/bin/openssl', 'pkey', '-in', str(signing_key),
                        '-pubout', '-outform', 'DER', '-out', str(root_key)], check=True)
        body, signature = scratch / 'manifest', scratch / 'signature'
        write_json(body, {'channel': CHANNEL, 'minimum_launcher': launcher_version, 'client': manifest,
                          'packages': release_packages(manifest['files'])})
        subprocess.run(['/usr/bin/openssl', 'pkeyutl', '-sign', '-rawin', '-inkey', str(signing_key),
                        '-in', str(body), '-out', str(signature)], check=True)
        envelope = {'payload': base64.b64encode(body.read_bytes()).decode('ascii'),
                    'signature': base64.b64encode(signature.read_bytes()).decode('ascii')}
        verify_envelope(envelope, (root_key,))
        launcher = downloads / f'theia-launcher_{launcher_version}_amd64.deb'
        package(launcher, launcher_version, (root_key,))
        with launcher.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        (downloads / 'SHA256SUMS').write_text(f'{digest}  {launcher.name}\n', encoding='ascii')
        (tree / '.nojekyll').touch()
        write_json(downloads / CHANNEL_FILE, envelope)
        if sum(path.stat().st_size for path in tree.rglob('*') if path.is_file()) > MAX_RELEASE_BYTES:
            raise InstallError('Pages artifact exceeds the size budget')
        tree.rename(output)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('client_export', type=Path, help=f'Linux release folder containing {MANIFEST}')
    parser.add_argument('output', type=Path)
    parser.add_argument('--launcher-version', required=True)
    parser.add_argument('--signing-key', required=True, type=Path)
    parser.add_argument('--retain-client', action='append', default=[], type=Path)
    args = parser.parse_args()
    try:
        print(stage(args.client_export, args.output, args.launcher_version, args.signing_key,
                    args.retain_client))
    except (OSError, InstallError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Could not stage the Pages downloads: {error}\n')


if __name__ == '__main__':
    main()
