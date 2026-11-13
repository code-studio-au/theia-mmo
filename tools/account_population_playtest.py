"""Owned account playtest for a checked editor population, with frozen development profiles."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import play

# Explicit, reviewed development setup. Normal client/server preflight remains
# responsible for validating every binding against the exported population.
PROFILES = [
    ('--abilities', 'development-classic-abilities.json'),
    ('--character-stats', 'development-character-stats.json'),
    ('--statuses', 'development-statuses.json'),
    ('--items', 'development-equipment-items.json'),
    ('--creature-retaliation', 'development-creature-retaliation.json'),
    ('--player-respawn', 'development-player-respawn.json'),
    ('--melee-weapons', 'development-melee-weapons.json'),
    ('--physical-definitions', 'development-physical-damage-definitions.json'),
    ('--physical-stats', 'development-physical-damage-stats.json'),
    ('--white-melee', 'development-white-melee.json'),
    ('--ability-starting', 'development-ability-starting.json'),
    ('--ability-damage', 'development-ability-damage.json'),
    ('--combat-membership', 'development-combat-membership.json'),
    ('--creature-credit', 'development-creature-credit-solo.json'),
    ('--progression-catalog', 'development-progression.json'),
    ('--reward-catalog', 'development-rewards.json'),
]


RESET_PROFILE = ('--creature-reset', 'development-creature-return-reset.json')
THREAT_PROFILE = ('--creature-threat', 'development-creature-threat.json')


def configured_pursuit(package):
    def document(path):
        with path.open('rb') as file:
            data = file.read(4097)
        if len(data) > 4096:
            raise ValueError('Population behavior input exceeds byte budget')
        value = json.loads(data)
        if not isinstance(value, dict):
            raise ValueError('Population behavior input must be an object')
        return value
    value = document(package/'population.json').get('creature')
    if not isinstance(value, str) or not value or len(value) > 256 or '\\' in value:
        raise ValueError('Invalid population creature path')
    relative = Path(value)
    if relative.is_absolute() or any(part in ('.', '..') for part in relative.parts):
        raise ValueError('Population creature path must stay inside the package')
    path = (package/relative).resolve()
    if not path.is_relative_to(package):
        raise ValueError('Population creature path escapes the package')
    return document(path).get('aggro') is not None


def prepare(args, repository):
    package, output = args.package.resolve(), args.output.resolve()
    players, binaries = args.players.resolve(), args.binaries.resolve()
    for path in [package/'population.json', package/'world/authored.world.json', players/'character.glb']:
        if not path.is_file():
            raise ValueError(f'Missing account playtest dependency: {path}')
    items = getattr(args, 'items', None)
    rewards = getattr(args, 'reward_catalog', None)
    if bool(items) != bool(rewards):
        raise ValueError('Select item and reward catalogues together')
    pursuit = configured_pursuit(package)
    overrides = {'--items': items, '--reward-catalog': rewards}
    profiles = []
    corpse_loot = False
    for flag, name in [*PROFILES, *([RESET_PROFILE, THREAT_PROFILE] if pursuit else [])]:
        selected = overrides.get(flag)
        source = selected.resolve() if selected else repository/'content/gameplay'/name
        if selected:
            name = 'selected-items.json' if flag == '--items' else 'selected-rewards.json'
        with source.open('rb') as file:
            data = file.read(1024 * 1024 + 1)
        if len(data) > 1024 * 1024:
            raise ValueError(f'Account playtest profile exceeds byte budget: {name}')
        if selected and flag == '--reward-catalog':
            document = json.loads(data)
            if not isinstance(document, dict) or not isinstance(document.get('rewards'), list):
                raise ValueError('Invalid reward catalogue')
            corpse_loot = any(isinstance(row, dict) and row.get('scope') == 'per_creature_life'
                              for row in document['rewards'])
        profiles.append((flag, name, data))
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    frozen = output/'profiles'
    frozen.mkdir(mode=0o700)
    command = ['--authored-world', str(package/'world'), '--population-config', str(package/'population.json'),
               '--players', str(players), '--binaries', str(binaries), '--window-size', args.window_size,
               '--classic', '--account-login', '--equipment', '--status-effects', '--progression', '--rewards',
               '--public-avatar-life', '--stop-on-stdin-eof', '--session-directory', str(output/'runtime')]
    if corpse_loot:
        command.append('--creature-loot')
    command.extend(['--character-id', args.character_id] if args.character_id else ['--character-picker'])
    if (package/'creature.glb').is_file():
        command.extend(['--creature-asset', str(package/'creature.glb')])
    records = []
    for flag, name, data in profiles:
        (frozen/name).write_bytes(data)
        command.extend([flag, str(frozen/name)])
        records.append(dict(option=flag, file=f'profiles/{name}', sha256=hashlib.sha256(data).hexdigest()))
    (output/'launch.json').write_text(json.dumps(dict(schema=1, package=str(package), profiles=records, creature_loot=corpse_loot, creature_reset=pursuit, creature_threat=pursuit,
        scope='One authenticated character on the exported population; development typed combat and durable grants; shared NPC retaliation and configured pursuit, return reset and damage threat; no Test Player controllers'), indent=2)+'\n')
    return command


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--players', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--window-size', default='1280x720')
    parser.add_argument('--items', type=Path, help='Explicit item catalogue, paired with --reward-catalog')
    parser.add_argument('--reward-catalog', type=Path, help='Explicit rewards; per-life entries enable ordinary corpse pickup')
    parser.add_argument('--character-id', help='Optional existing owned character; otherwise publish the private picker')
    args = parser.parse_args()
    repository = Path(__file__).resolve().parent.parent
    command = prepare(args, repository)
    # Keep the normal launcher's repository-relative development dependencies
    # consistent even when the editor was opened from another working directory.
    os.chdir(repository)
    play.main(command)


if __name__ == '__main__':
    main()
