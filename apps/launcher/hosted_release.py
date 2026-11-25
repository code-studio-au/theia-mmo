"""Signed Development releases served as immutable files over HTTPS."""
import base64
from dataclasses import dataclass
import hashlib
import http.client
import json
from pathlib import Path
import subprocess
import tempfile
from urllib.error import URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

from client_install import (
    CHUNK_BYTES, MAX_MANIFEST_BYTES, InstallError, active, copy_file, decode_json,
    install_release, installed, regular, validate_metadata, version_key,
)

# A separately deployed Pages project keeps packages live across website builds.
DEFAULT_URL = 'https://code-studio-au.github.io/theia-mmo/downloads/'
CHANNEL_FILE = 'development.json'
CHANNEL = 'Development'
MAX_ENVELOPE = 6 * 1024**2
MAX_RELEASE_BYTES = 950_000_000
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


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


@dataclass(frozen=True)
class HostedRelease:
    library: 'HostedLibrary'
    manifest: dict
    total: int

    def download_size(self, previous_files=()):
        known = {content_key(row) for row in previous_files}
        missing = {content_key(row) for row in self.manifest['files']} - known
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
        for entry in cache.iterdir():
            if entry.name not in digests:
                entry.unlink()

        current = active(root)['current']
        old_directory, old_files = None, {}
        if current:
            old_directory, manifest, _ = installed(root, current)
            old_files = {content_key(row): row['path'] for row in manifest['files']}

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
            if not reused:
                partial = cache / row['sha256']
                self.library.download(row, partial, advance)
                if copy_file(partial, destination, row['bytes'], lambda _: None) != (
                        row['bytes'], row['sha256']):
                    partial.unlink()
                    raise InstallError(f'Download verification failed: {name}')
            advance(row['bytes'])
            completed += row['bytes']

        # Only verified bytes reach this point. Activation remains under the installer lock.
        for entry in cache.iterdir():
            entry.unlink()


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
        except (URLError, TimeoutError) as error:
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
        release = HostedRelease(self, manifest, total)
        previous_files = ()
        if current and install_root is not None:
            _, previous, _ = installed(install_root, current)
            previous_files = previous['files']
        return release, manifest['version'], release.download_size(previous_files)

    def download(self, row, partial, progress):
        size = row['bytes']
        offset = regular(partial).stat().st_size if partial.exists() or partial.is_symlink() else 0
        if offset > size:
            partial.unlink()
            offset = 0
        if size == 0:
            partial.touch()
            return
        if offset == size:
            return
        with self.open('blobs/' + row['sha256'], offset=offset) as response:
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
                except (http.client.HTTPException, TimeoutError) as error:
                    raise InstallError('The download was interrupted. Try again to resume.') from error
        if offset != size:
            raise InstallError('The download was interrupted. Try again to resume.')


def verify_envelope(envelope, public_keys):
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
    data = decode_json(payload)
    fields = {'channel', 'minimum_launcher', 'client'}
    if (not isinstance(data, dict) or set(data) not in (fields, fields | {'packages'})
            or data['channel'] != CHANNEL):
        raise InstallError('Unknown hosted release format or channel')
    version_key(data['minimum_launcher'])
    return data
