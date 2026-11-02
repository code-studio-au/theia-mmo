"""Launch-time exact-byte snapshot; gameplay schemas belong to the Rust validator."""
from contextlib import ExitStack, contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from launch_environment import client_environment


ROLES = frozenset({'items', 'abilities', 'stats', 'statuses', 'creature',
                   'dialogue', 'vendor', 'rewards', 'progression', 'quest'})
REWARDED_ROLES = ROLES | {'quest_turn_in_xp'}
LIFE_ROLES = REWARDED_ROLES | {'creature_retaliation', 'player_respawn'}
DODGE_ROLES = LIFE_ROLES | {'dodge'}
MELEE_ROLES = LIFE_ROLES | {'melee_weapons'}
PHYSICAL_ROLES = MELEE_ROLES | {'physical_definitions', 'physical_stats'}
WHITE_ROLES = PHYSICAL_ROLES | {'white_melee'}
ABILITY_ROLES = WHITE_ROLES | {'ability_starting', 'ability_damage'}
MEMBERSHIP_ROLES = ABILITY_ROLES | {'combat_membership'}
CREDIT_ROLES = MEMBERSHIP_ROLES | {'creature_credit'}
RESET_ROLES = CREDIT_ROLES | {'creature_reset'}
MAX_PROFILE_BYTES = 64 * 1024  # Outer copy bound; Rust enforces each smaller role budget.
MAX_MANIFEST_BYTES = 8192
MAX_REPORT_BYTES = 16384


class BundleError(ValueError):
    pass


def _read(path, limit):
    if not path.is_file():
        raise BundleError('Gameplay bundle input must be a regular file')
    with path.open('rb') as stream:
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise BundleError('Gameplay bundle input exceeds copy budget')
    return data


def _validate(validator, manifest):
    # The executable is a trusted local build. Capture bounded output without
    # echoing raw invalid content or provider/database environment to the tool.
    with tempfile.TemporaryFile() as output:
        result = subprocess.run([str(validator), str(manifest)], stdout=output,
                                stderr=subprocess.DEVNULL, timeout=30,
                                env={k: v for k, v in client_environment(os.environ).items()
                                     if not k.upper().startswith('THEIA_')})
        output.seek(0)
        raw = output.read(MAX_REPORT_BYTES + 1)
    if result.returncode or len(raw) > MAX_REPORT_BYTES:
        raise BundleError('Gameplay manifest validation failed')
    try:
        report = json.loads(raw)
        records = report['profiles']
        manifest_schema = report.get('manifest_schema', 1)
        roles = {1: ROLES, 2: REWARDED_ROLES, 3: LIFE_ROLES, 4: DODGE_ROLES, 5: MELEE_ROLES, 6: PHYSICAL_ROLES, 7: WHITE_ROLES, 8: ABILITY_ROLES, 9: MEMBERSHIP_ROLES, 10: CREDIT_ROLES, 11: RESET_ROLES}.get(manifest_schema, ())
        if (report['validated'] is not True or report['schema'] != 1
                or type(manifest_schema) is not int or manifest_schema not in (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
                or len(records) != len(roles)
                or {r['kind'] for r in records} != roles
                or not re.fullmatch('[0-9a-f]{64}', report['manifest_sha256'])):
            raise ValueError()
        for record in records:
            name, size, digest = record['path'], record['bytes'], record['sha256']
            if (not isinstance(name, str) or len(name) > 256
                    or not re.fullmatch('[A-Za-z0-9_./-]+[.]json', name)
                    or any(part in ('', '.', '..') for part in name.split('/'))
                    or type(size) is not int or not 0 < size <= MAX_PROFILE_BYTES
                    or not re.fullmatch('[0-9a-f]{64}', digest)):
                raise ValueError()
    except (ValueError, KeyError, TypeError):
        raise BundleError('Gameplay validator returned an invalid report') from None
    return report


def _matching(data, digest):
    if hashlib.sha256(data).hexdigest() != digest:
        raise BundleError('Gameplay inputs changed during validation; retry launch')


@contextmanager
def snapshot(manifest, validator, session):
    """Yield (role paths, report), retaining files until both game children stop.

    Read/hash each bounded copy against Rust's report, then revalidate the copied
    manifest with Rust. Later edits to authoring files cannot split client/server
    content. This does not snapshot terrain, navmesh, assets or spell feedback.
    """
    with ExitStack() as stack:
        try:
            manifest = manifest.resolve(strict=True)
            validator = validator.resolve(strict=True)
            report = _validate(validator, manifest)
            raw = _read(manifest, MAX_MANIFEST_BYTES)
            _matching(raw, report['manifest_sha256'])
            session.mkdir(parents=True, exist_ok=True)
            folder = stack.enter_context(tempfile.TemporaryDirectory(prefix='gameplay-', dir=session))
            root = Path(folder).resolve()
            copied_manifest = root / manifest.name
            copied_manifest.write_bytes(raw)
            paths = {}
            for record in report['profiles']:
                source = (manifest.parent / record['path']).resolve(strict=True)
                if not source.is_relative_to(manifest.parent):
                    raise BundleError('Gameplay profile escapes its content directory')
                data = _read(source, MAX_PROFILE_BYTES)
                _matching(data, record['sha256'])
                if len(data) != record['bytes']:
                    raise BundleError('Gameplay profile byte count changed')
                dest = root / record['path']
                if dest == copied_manifest:
                    raise BundleError('Gameplay profile aliases snapshot manifest')
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(data)
                paths[record['kind']] = dest
            if _validate(validator, copied_manifest) != report:
                raise BundleError('Gameplay snapshot differs from validated input')
        except (OSError, subprocess.SubprocessError) as error:
            raise BundleError('Gameplay snapshot unavailable; check validator and content') from error
        yield paths, report
