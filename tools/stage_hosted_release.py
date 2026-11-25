"""Prepare an approved Linux client export and launcher for GitHub Pages."""
import argparse
import base64
import hashlib
import shutil
from pathlib import Path
import subprocess
import tempfile

from client_install import MANIFEST, InstallError, copy_file, metadata, version, version_key, write_json
from package_client_launcher import package
from retain_pages_release import STABLE_LAUNCHER, retain_previous
from release_news import NEWS_FILE, NEWS_LIMIT, next_archive
from hosted_release import (
    CHANNEL, CHANNEL_FILE, DEFAULT_URL, DELTA_LIMIT, MAX_RELEASE_BYTES, HostedLibrary,
    ZSTD, apply_delta, delta_eligible, release_packages, validate_deltas, verified_payload, verify_envelope,
)


def sign(payload, scratch, key):
    with tempfile.TemporaryDirectory(prefix='sign-', dir=scratch) as directory:
        body, signature = Path(directory) / 'payload', Path(directory) / 'signature'
        write_json(body, payload)
        subprocess.run(['/usr/bin/openssl', 'pkeyutl', '-sign', '-rawin', '-inkey', str(key),
                        '-in', str(body), '-out', str(signature)], check=True)
        return {'payload': base64.b64encode(body.read_bytes()).decode('ascii'),
                'signature': base64.b64encode(signature.read_bytes()).decode('ascii')}


def build_deltas(base, candidate, blobs, scratch, budget):
    """Build useful patches for one verified older export while keeping full target blobs."""
    base = base.resolve(strict=True)
    previous, _ = metadata(base)
    if (previous['target'] != candidate['target']
            or version_key(previous['version']) >= version_key(candidate['version'])):
        raise InstallError('Choose an older binary patch base for the same platform')
    if not ZSTD.is_file():
        raise InstallError('Install zstd to publish binary patches')
    old_files = {row['path']: row for row in previous['files']}
    entries = []
    for row in sorted(candidate['files'], key=lambda entry: (-entry['bytes'], entry['path'])):
        if len(entries) >= DELTA_LIMIT or budget <= 0:
            break
        old = old_files.get(row['path'])
        if not delta_eligible(old, row):
            continue
        source = base / 'payload' / row['path']
        if copy_file(source, None, old['bytes'], lambda _: None) != (old['bytes'], old['sha256']):
            raise InstallError(f'Binary patch base failed verification: {row["path"]}')
        with tempfile.TemporaryDirectory(prefix='delta-', dir=scratch) as directory:
            patch = Path(directory) / 'patch.zst'
            try:
                subprocess.run([ZSTD, '-q', '-T1', '-3', f'--patch-from={source}',
                                '-o', patch, '--', blobs / row['sha256']],
                               check=True, timeout=45, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL)
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
                continue
            size = patch.stat().st_size
            if not 0 < size * 5 <= row['bytes'] * 4:
                continue
            with patch.open('rb') as stream:
                digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            destination = blobs / digest
            charged = 0 if destination.exists() else size
            if charged > budget:
                continue
            try:
                apply_delta(source, patch, Path(directory) / 'rebuilt', row, lambda _: None)
            except InstallError:
                continue
            if charged:
                patch.rename(destination)
                budget -= charged
            entries.append({'path': row['path'], 'base_sha256': old['sha256'],
                            'base_bytes': old['bytes'], 'target_sha256': row['sha256'],
                            'sha256': digest, 'bytes': size})
    return {'format': 'zstd-patch-v1', 'version': candidate['version'], 'entries': entries}


def stage(release, output, launcher_version, signing_key, retained=(), *, delta_base=None,
          live_mode=None, live_url=DEFAULT_URL, live_opener=None, expected_previous=None, release_notes=None):
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
        envelope = sign({'channel': CHANNEL, 'minimum_launcher': launcher_version, 'client': manifest,
                         'packages': release_packages(manifest['files'])}, scratch, signing_key)
        verify_envelope(envelope, (root_key,))
        launcher = downloads / f'theia-launcher_{launcher_version}_amd64.deb'
        package(launcher, launcher_version, (root_key,))
        previous = None
        if live_mode is not None:
            live = HostedLibrary(live_url, (root_key,), opener=live_opener)
            previous = retain_previous(live, downloads, manifest, launcher_version,
                                       first_release=live_mode == 'first', expected_previous=expected_previous)
            news = next_archive(live, previous, launcher_version, manifest['version'], release_notes or {})
            news_path = downloads / NEWS_FILE
            write_json(news_path, sign(news, scratch, signing_key))
            if news_path.stat().st_size > NEWS_LIMIT:
                raise InstallError('The signed release news archive exceeds its size limit')
        shutil.copyfile(launcher, downloads / STABLE_LAUNCHER)
        sums = []
        for installer in sorted(downloads.glob('theia-launcher_*.deb')):
            with installer.open('rb') as stream:
                sums.append(f"{hashlib.file_digest(stream, 'sha256').hexdigest()}  {installer.name}\n")
        (downloads / 'SHA256SUMS').write_text(''.join(sums), encoding='ascii')
        (tree / '.nojekyll').touch()
        write_json(downloads / CHANNEL_FILE, envelope)
        if delta_base is not None and (previous is None or previous['client'] != manifest):
            used = sum(path.stat().st_size for path in tree.rglob('*') if path.is_file())
            # Leave space for the signed sidecar even at the artifact limit.
            payload = build_deltas(delta_base, manifest, blobs, scratch, MAX_RELEASE_BYTES - used - 512 * 1024)
            if payload['entries']:
                signed = sign(payload, scratch, signing_key)
                validate_deltas(verified_payload(signed, (root_key,)), manifest)
                (downloads / 'deltas').mkdir(exist_ok=True)
                write_json(downloads / 'deltas' / f"{manifest['version']}.json", signed)
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
    parser.add_argument('--delta-base', type=Path, help='Verified older client export to generate binary patches from')
    parser.add_argument('--live-mode', choices=('first', 'update'),
                        help='Check the public feed and retain the live release before publishing')
    parser.add_argument('--expected-previous', help='Exact live client version in update mode')
    parser.add_argument('--launcher-notes', default='', help='Reviewed launcher changes, one per line')
    parser.add_argument('--client-notes', default='', help='Reviewed client changes, one per line')
    args = parser.parse_args()
    try:
        print(stage(args.client_export, args.output, args.launcher_version, args.signing_key,
                    args.retain_client, delta_base=args.delta_base, live_mode=args.live_mode,
                    expected_previous=version(args.expected_previous) if args.expected_previous else None,
                    release_notes={'launcher': args.launcher_notes, 'client': args.client_notes}))
    except (OSError, InstallError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Could not stage the Pages downloads: {error}\n')


if __name__ == '__main__':
    main()
