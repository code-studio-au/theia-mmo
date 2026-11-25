"""Carry the live release forward before replacing a GitHub Pages artifact."""
import hashlib
import re
from urllib.error import HTTPError

from client_install import InstallError, copy_file, decode_json, validate_metadata, version_key
from hosted_release import (
    CHANNEL_FILE, MAX_BUNDLE_METADATA, MAX_ENVELOPE, MAX_RELEASE_BYTES, validate_bundles,
    validate_deltas, verified_payload, verify_envelope,
)

STABLE_LAUNCHER = 'theia-launcher_amd64.deb'
MAX_INSTALLER = 64 * 1024**2


def read_remote(library, name, limit):
    try:
        with library.open(name) as response:
            data = response.read(limit + 1)
    except InstallError as error:
        if isinstance(error.__cause__, HTTPError) and error.__cause__.code == 404:
            return None
        raise
    if len(data) > limit:
        raise InstallError(f'Live release asset exceeds its size limit: {name}')
    return data


def retain_previous(library, downloads, current, launcher_version, *, first_release, expected_previous=None):
    raw = read_remote(library, CHANNEL_FILE, MAX_ENVELOPE)
    if raw is None:
        if first_release:
            return
        raise InstallError('Live release feed is missing; first-release mode must be explicit')
    if first_release:
        raise InstallError('A live release already exists; use update mode')
    previous = verify_envelope(decode_json(raw), library.public_keys)
    previous_manifest, _ = validate_metadata(previous['client'])
    if expected_previous is not None and previous_manifest['version'] != expected_previous:
        raise InstallError('The live client does not match the expected previous release')
    old_launcher = previous['minimum_launcher']
    old_client_key, new_client_key = (version_key(item['version']) for item in (previous_manifest, current))
    old_launcher_key, new_launcher_key = map(version_key, (old_launcher, launcher_version))
    if old_client_key > new_client_key or old_launcher_key > new_launcher_key:
        raise InstallError('Client and launcher releases cannot go backwards')
    if old_client_key == new_client_key:
        if previous_manifest != current:
            raise InstallError('Changed client files require a new client version')
        if old_launcher_key == new_launcher_key:
            raise InstallError('Advance the launcher or client version before publishing')

    installer = f'theia-launcher_{old_launcher}_amd64.deb'
    checksums = read_remote(library, 'SHA256SUMS', 8192)
    pattern = rb'([a-f0-9]{64})  ' + re.escape(installer.encode('ascii'))
    matches = [m.group(1).decode() for line in (checksums or b'').splitlines()
               if (m := re.fullmatch(pattern, line))]
    if len(matches) != 1:
        raise InstallError('Live launcher checksum is missing or ambiguous')
    old_digest = matches[0]

    delta_path = f"deltas/{previous_manifest['version']}.json"
    delta_raw = read_remote(library, delta_path, MAX_ENVELOPE)
    delta_rows = []
    if delta_raw is not None:
        delta_rows = list(validate_deltas(verified_payload(decode_json(delta_raw), library.public_keys),
                                         previous_manifest).values())
    bundle_path = f"bundles/{previous_manifest['version']}.json"
    bundle_raw = read_remote(library, bundle_path, MAX_BUNDLE_METADATA)
    bundle_rows = []
    if bundle_raw is not None:
        bundle_rows = list(validate_bundles(verified_payload(decode_json(bundle_raw), library.public_keys),
                                           previous_manifest).values())

    used = sum(path.stat().st_size for path in downloads.rglob('*') if path.is_file())
    seen = set()
    for row in previous_manifest['files'] + delta_rows + bundle_rows:
        digest = row['sha256']
        if digest in seen:
            continue
        seen.add(digest)
        blob = downloads / 'blobs' / digest
        if not blob.exists():
            used += row['bytes']
            if used > MAX_RELEASE_BYTES:
                raise InstallError('The current and previous releases exceed the Pages size budget')
            library.download(row, blob, lambda _: None)
        if copy_file(blob, None, row['bytes'], lambda _: None) != (row['bytes'], digest):
            raise InstallError(f'Live release blob failed verification: {digest}')

    destination = downloads / installer
    if destination.exists():
        # A client-only release may reuse the exact published launcher. Keep
        # its immutable bytes, including the original package timestamps.
        destination.unlink()
    data = read_remote(library, installer, MAX_INSTALLER)
    if data is None or hashlib.sha256(data).hexdigest() != old_digest:
        raise InstallError('The live launcher download failed verification')
    destination.write_bytes(data)
    for name, raw_data in ((delta_path, delta_raw), (bundle_path, bundle_raw)):
        if raw_data is not None:
            path = downloads / name
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(raw_data)

    return previous
