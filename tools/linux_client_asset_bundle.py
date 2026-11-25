"""Freeze reviewed files from a Linux client export for the candidate build."""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import tarfile
import tempfile

from client_install import InstallError, MAX_FILES, copy_file, metadata, regular, relative


def asset_path(value):
    path = relative(value)
    if path.casefold() in {'theia-client', 'theia-client.exe'}:
        raise InstallError('The asset bundle must not contain a client executable')
    return path


def pack(release, paths, output):
    release, output = release.resolve(strict=True), output.absolute()
    manifest, _ = metadata(release)
    selected = sorted(asset_path(path) for path in paths)
    if not selected or len(selected) > MAX_FILES or len(set(selected)) != len(selected):
        raise InstallError('Choose a nonempty list of distinct reviewed asset paths')
    files = {row['path']: row for row in manifest['files']}
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.linux-assets-', dir=output.parent) as directory:
        scratch = Path(directory)
        packed, staged = scratch / 'assets.tar.gz', scratch / 'file'
        with packed.open('xb') as raw, gzip.GzipFile(filename='', fileobj=raw, mode='wb', mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode='w|', format=tarfile.PAX_FORMAT) as archive:
                for path in selected:
                    if path not in files:
                        raise InstallError(f'Asset is absent from the verified export: {path}')
                    entry = files[path]
                    size, digest = copy_file(release / 'payload' / path, staged, entry['bytes'], lambda _: None)
                    if (size, digest) != (entry['bytes'], entry['sha256']):
                        raise InstallError(f'Asset differs from the verified export: {path}')
                    member = tarfile.TarInfo(path)
                    member.size = size
                    member.mode = 0o644
                    with staged.open('rb') as stream:
                        archive.addfile(member, stream)
                    staged.unlink()
        with packed.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        size = packed.stat().st_size
        os.link(packed, output)
    return {'bytes': size, 'sha256': digest}


def unpack(bundle, approval, staging, remaining):
    """Check the pinned archive before copying regular files into the payload."""
    with tempfile.TemporaryDirectory(prefix='.linux-bundle-', dir=staging.parent) as directory:
        pinned = Path(directory) / 'assets.tar.gz'
        observed = copy_file(bundle, pinned, approval['bytes'], lambda _: None)
        if observed != (approval['bytes'], approval['sha256']):
            raise InstallError('Linux asset bundle differs from the approved bytes')
        count = 0
        with tarfile.open(pinned, mode='r|gz') as archive:
            for member in archive:
                path = asset_path(member.name)
                count += 1
                if member.type != tarfile.REGTYPE or not 0 <= member.size <= remaining or count >= MAX_FILES:
                    raise InstallError(f'Invalid Linux asset bundle entry: {path}')
                destination = staging / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                if destination.exists():
                    raise InstallError(f'Duplicate Linux asset bundle entry: {path}')
                with archive.extractfile(member) as source, destination.open('xb') as output:
                    shutil.copyfileobj(source, output, length=1024 * 1024)
                remaining -= member.size
        if not count:
            raise InstallError('Linux asset bundle is empty')
    return remaining


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('release', type=Path)
    parser.add_argument('paths', type=Path, help='Reviewed payload paths, one per line')
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    try:
        with regular(args.paths.absolute()).open(encoding='utf-8') as stream:
            paths = stream.read(MAX_FILES * 241 + 1)
        if len(paths) > MAX_FILES * 241:
            raise InstallError('Reviewed asset path list exceeds its size limit')
        print(json.dumps(pack(args.release, paths.splitlines(), args.output), sort_keys=True))
    except (InstallError, OSError, UnicodeError, tarfile.TarError) as error:
        parser.exit(1, f'Could not pack Linux assets: {error}\n')


if __name__ == '__main__':
    main()
