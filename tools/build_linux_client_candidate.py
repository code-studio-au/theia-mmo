"""Export a Linux client with a reviewed list of redistributable payload files."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from client_install import InstallError, copy_file, export, read_json, relative, version
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps' / 'launcher'))
from hosted_release import MAX_RELEASE_BYTES


def build(repo, client, approval, output, release_version):
    repo, output = repo.resolve(strict=True), output.resolve()
    version(release_version)
    approval = approval.absolute()
    data = read_json(approval, 1024 * 1024)
    if not isinstance(data, dict) or set(data) != {'files'} or not isinstance(data['files'], list) or not data['files']:
        raise InstallError('A reviewed public asset list is required')
    tracked = set(subprocess.check_output(['git', '-C', repo, 'ls-files', '-z']).decode().split('\0'))
    if approval.resolve().relative_to(repo).as_posix() not in tracked:
        raise InstallError('The public asset approval must be checked into the repository')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='linux-client-', dir=output.parent) as folder:
        staging = Path(folder)
        client = client.resolve(strict=True)
        remaining = MAX_RELEASE_BYTES
        client_size = client.stat().st_size
        with client.open('rb') as stream:
            header = stream.read(20)
        if (header[:6] != b'\x7fELF\x02\x01' or len(header) != 20
                or int.from_bytes(header[18:20], 'little') != 62
                or not 0 < client_size < remaining):
            raise InstallError('Expected a built Linux AMD64 client')
        copy_file(client, staging / 'theia-client', client_size, lambda _: None)
        remaining -= client_size
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
    args = parser.parse_args()
    try:
        print(build(Path.cwd(), args.client, args.approval, args.output, args.version))
    except (OSError, InstallError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Could not build the release candidate: {error}\n')


if __name__ == '__main__':
    main()
