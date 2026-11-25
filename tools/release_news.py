"""Build an append-only, signed release-news archive for the public website."""
from datetime import UTC, datetime

from client_install import InstallError, decode_json, version, version_key
from hosted_release import verified_payload
from retain_pages_release import read_remote

NEWS_FILE = 'release-news.json'
NEWS_LIMIT = 1024 * 1024
MAX_ENTRIES = 256
PRODUCTS = ('launcher', 'client')


def lines(text):
    result = [line.strip() for line in text.splitlines() if line.strip()]
    if (not 1 <= len(result) <= 12
            or any(not 1 <= len(line) <= 280 or not line.isprintable() for line in result)):
        raise InstallError('Use 1–12 plain-text release notes of at most 280 characters per line')
    return result


def validate_archive(data):
    if (not isinstance(data, dict) or set(data) != {'schema', 'entries'} or type(data['schema']) is not int
            or data['schema'] != 1 or not isinstance(data['entries'], list)
            or len(data['entries']) > MAX_ENTRIES):
        raise InstallError('Invalid release news archive')
    newest = {}
    for row in data['entries']:
        if (not isinstance(row, dict) or set(row) != {'product', 'version', 'date', 'notes'}
                or row['product'] not in PRODUCTS
                or not isinstance(row['notes'], list)
                or any(not isinstance(line, str) for line in row['notes'])
                or lines('\n'.join(row['notes'])) != row['notes']):
            raise InstallError('Invalid release news entry')
        match_version = version(row['version'])
        try:
            date = datetime.strptime(row['date'], '%Y-%m-%dT%H:%M:%SZ')
        except (ValueError, TypeError) as error:
            raise InstallError('Invalid release news date') from error
        if date.strftime('%Y-%m-%dT%H:%M:%SZ') != row['date']:
            raise InstallError('Invalid release news date')
        product = row['product']
        if product in newest and version_key(newest[product]) >= version_key(match_version):
            raise InstallError('Duplicate or out-of-order release news')
        newest[product] = match_version
    return newest


def next_archive(live, previous, launcher_version, client_version, notes, *, now=None):
    archive = {'schema': 1, 'entries': []}
    before = {}
    if previous is not None:
        raw = read_remote(live, NEWS_FILE, NEWS_LIMIT)
        if raw is None:
            raise InstallError('The live signed release news archive is missing')
        archive = verified_payload(decode_json(raw), live.public_keys)
        before = {'launcher': previous['minimum_launcher'], 'client': previous['client']['version']}
        if validate_archive(archive) != before:
            raise InstallError('The live release news does not match the live download versions')

    after = {'launcher': launcher_version, 'client': client_version}
    date = (now or datetime.now(UTC)).strftime('%Y-%m-%dT%H:%M:%SZ')
    for product in PRODUCTS:
        if (product in before and version_key(before[product]) == version_key(after[product])):
            if notes.get(product, '').strip():
                raise InstallError(f'Leave {product} release notes blank when its version is unchanged')
            continue
        if not notes.get(product, '').strip():
            raise InstallError(f'Provide reviewed {product} release notes for {after[product]}')
        archive['entries'].append({'product': product, 'version': after[product],
                                   'date': date, 'notes': lines(notes[product])})
    validate_archive(archive)
    return archive
