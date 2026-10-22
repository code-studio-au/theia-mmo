#!/usr/bin/env python3
"""Inspect/extract explicitly selected Unity source payloads without running code."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tarfile

MAX_MEMBER = 128 * 1024 * 1024
MAX_SELECTED = 512 * 1024 * 1024


def asset_path(value):
    # Some downloaded packages append a newline/00 trailer to pathname records.
    lines = value.decode('utf-8').splitlines()
    if not lines or any(line not in ('', '00', '\0') for line in lines[1:]):
        raise ValueError('Unsupported Unity pathname trailer')
    name = lines[0].rstrip('\0')
    path = PurePosixPath(name)
    if (not name.startswith('Assets/') or path.is_absolute() or '\\' in name or ':' in name
            or any(part in ('.', '..', '') for part in name.split('/')) or any(ord(c) < 32 for c in name)):
        raise ValueError('Unsafe Unity asset path')
    return name


def members(archive, include_meta=False):
    # gzip handles the optional extra header that Python tarfile's stream gzip
    # reader mishandles for some Unity packages. Never call tar.extractall.
    with gzip.open(archive, 'rb') as source, tarfile.open(fileobj=source, mode='r|') as stream:
        for member in stream:
            parts = member.name.split('/')
            if len(parts) != 2 or not re.fullmatch('[0-9a-f]{32}', parts[0]):
                continue
            if member.issym() or member.islnk():
                raise ValueError('Unity payload aliases are unsupported')
            if member.isfile() and parts[1] in (('pathname', 'asset', 'asset.meta') if include_meta else ('pathname', 'asset')):
                yield stream, parts[0], parts[1], member


def inspect(archive):
    paths, sizes = {}, {}
    for stream, guid, kind, member in members(archive):
        if kind == 'pathname':
            if guid in paths or not 1 <= member.size <= 16384:
                raise ValueError('Duplicate or oversized Unity pathname')
            paths[guid] = asset_path(stream.extractfile(member).read())
        else:
            if guid in sizes or not 0 <= member.size <= MAX_MEMBER:
                raise ValueError('Duplicate or oversized Unity asset')
            sizes[guid] = member.size
        if len(paths) > 65536 or len(sizes) > 65536:
            raise ValueError('Unity package inventory exceeds budget')
    files = {path: {'guid': guid, 'bytes': sizes[guid]} for guid, path in paths.items() if guid in sizes}
    if len(files) != sum(guid in sizes for guid in paths):
        raise ValueError('Duplicate Unity asset destinations')
    return files


def extract_selected(archive, output, selected, include_meta=False):
    archive, output = Path(archive).resolve(strict=True), Path(output).resolve()
    if output.exists() or output == archive or archive.is_relative_to(output):
        raise FileExistsError('Extraction output must be new and outside source')
    selected = list(selected)
    if not selected or len(selected) > 4096 or len(set(selected)) != len(selected):
        raise ValueError('Select distinct explicit asset paths')
    for name in selected:
        if asset_path(name.encode()) != name:
            raise ValueError('Noncanonical selection')
    if include_meta and set(selected) & {name + '.meta' for name in selected}:
        raise ValueError('Selected payload/metadata destinations overlap')
    before = hashlib.sha256(archive.read_bytes()).hexdigest()
    inventory = inspect(archive)
    if any(name not in inventory for name in selected):
        raise ValueError('Selected Unity asset is missing')
    if sum(inventory[name]['bytes'] for name in selected) > MAX_SELECTED:
        raise ValueError('Selected Unity payload exceeds budget')
    wanted = {inventory[name]['guid']: name for name in selected}
    output.mkdir(parents=True)
    report, metadata, total = {}, {}, 0
    try:
        for stream, guid, kind, member in members(archive, include_meta):
            if kind not in ('asset', 'asset.meta') or guid not in wanted:
                continue
            name = wanted[guid]
            target_report = report if kind == 'asset' else metadata
            if (kind == 'asset' and member.size != inventory[name]['bytes']) or name in target_report or member.size > MAX_MEMBER:
                raise ValueError('Unity package changed during extraction')
            total += member.size
            if total > MAX_SELECTED:
                raise ValueError('Selected Unity payload/metadata exceeds budget')
            target = output / (name + ('.meta' if kind == 'asset.meta' else ''))
            target.parent.mkdir(parents=True, exist_ok=True)
            digest = hashlib.sha256()
            with stream.extractfile(member) as source, target.open('xb') as destination:
                while data := source.read(1024 * 1024):
                    digest.update(data)
                    destination.write(data)
            target_report[name] = {'guid': guid, 'bytes': member.size, 'sha256': digest.hexdigest()}
        if include_meta and set(metadata) != set(selected):
            raise ValueError('Selected Unity metadata is missing')
        if set(report) != set(selected) or hashlib.sha256(archive.read_bytes()).hexdigest() != before:
            raise ValueError('Unity package changed during extraction')
        result = {'archive_sha256': before, 'selected': report, 'metadata': metadata, 'scope': 'source payload extraction only; no script execution or runtime import'}
        (output / 'extraction.json').write_text(json.dumps(result, indent=2) + '\n')
        return result
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--asset', action='append', default=[])
    parser.add_argument('--include-meta', action='store_true')
    args = parser.parse_args()
    result = extract_selected(args.archive, args.output, args.asset, args.include_meta) if args.output else inspect(args.archive)
    print(json.dumps(result, indent=2))
