"""Export a Linux client with a reviewed list of redistributable payload files."""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

from client_install import InstallError, copy_file, export, read_json, relative, version
from linux_client_asset_bundle import unpack
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps' / 'launcher'))
from hosted_release import MAX_RELEASE_BYTES


def build(repo, client, approval, output, release_version, *, asset_bundle=None):
    repo, output = repo.resolve(strict=True), output.resolve()
    version(release_version)
    approval = approval.absolute()
    data = read_json(approval, 1024 * 1024)
    if (not isinstance(data, dict) or set(data) not in ({'files'}, {'files', 'bundle'})
            or not isinstance(data['files'], list) or not (data['files'] or 'bundle' in data)):
        raise InstallError('A reviewed public asset list is required')
    bundle = data.get('bundle')
    if 'bundle' in data and (not isinstance(bundle, dict) or set(bundle) != {'bytes', 'sha256'}
            or type(bundle['bytes']) is not int or not 0 < bundle['bytes'] <= MAX_RELEASE_BYTES
            or not isinstance(bundle['sha256'], str) or not re.fullmatch('[a-f0-9]{64}', bundle['sha256'])):
        raise InstallError('Invalid reviewed Linux asset bundle')
    if (bundle is None) != (asset_bundle is None):
        raise InstallError('Supply the Linux asset bundle pinned in the reviewed list')
    tracked = set(subprocess.check_output(['git', '-C', repo, 'ls-files', '-z']).decode().split('\0'))
    if approval.resolve().relative_to(repo).as_posix() not in tracked:
        raise InstallError('The public asset approval must be checked into the repository')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='linux-client-', dir=output.parent) as folder:
        staging = Path(folder)
        client = client.resolve(strict=True)
        with client.open('rb') as stream:
            header = stream.read(20)
        if (header[:6] != b'\x7fELF\x02\x01' or len(header) != 20
                or int.from_bytes(header[18:20], 'little') != 62):
            raise InstallError('Expected a built Linux AMD64 client')
        executable = staging / 'theia-client'
        subprocess.run(['strip', '--strip-all', '-o', executable, '--', client], check=True)
        remaining = MAX_RELEASE_BYTES - executable.stat().st_size
        if remaining <= 0:
            raise InstallError('The stripped Linux client exceeds the release size budget')
        if asset_bundle is not None:
            remaining = unpack(asset_bundle.absolute(), bundle, staging, remaining)
        for row in data['files']:
            if not isinstance(row, dict) or set(row) != {'source', 'path', 'bytes', 'sha256'}:
                raise InstallError('Invalid public asset approval entry')
            source, dest = relative(row['source']), relative(row['path'])
            size, digest = row['bytes'], row['sha256']
            if (source not in tracked or not source.startswith(('content/', 'apps/client/assets/', 'release/'))
                    or dest == 'theia-client' or type(size) is not int or not 0 <= size <= remaining
                    or not isinstance(digest, str) or len(digest) != 64):
                raise InstallError(f'Public asset was not approved or exceeds the size budget: {source}')
            asset = repo / source
            if asset.resolve(strict=True) != asset:
                raise InstallError(f'Public asset must be a regular repository file: {source}')
            path = staging / dest
            path.parent.mkdir(parents=True, exist_ok=True)
            if copy_file(asset, path, size, lambda _: None) != (size, digest):
                raise InstallError(f'Public asset differs from the reviewed bytes: {source}')
            remaining -= size
        return export(staging, output, release_version)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--client', type=Path, default=Path('target/release/theia-client'))
    parser.add_argument('--approval', type=Path, default=Path('release/linux-client-files.json'))
    parser.add_argument('--asset-bundle', type=Path, help='Path to the reviewed asset bundle')
    args = parser.parse_args()
    try:
        print(build(Path.cwd(), args.client, args.approval, args.output, args.version, asset_bundle=args.asset_bundle))
    except (OSError, ValueError, subprocess.CalledProcessError, tarfile.TarError) as error:
        parser.exit(1, f'Could not build the release candidate: {error}\n')


if __name__ == '__main__':
    main()
