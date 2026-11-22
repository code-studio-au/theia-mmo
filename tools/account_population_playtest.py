"""Owned account playtest for a checked editor population, with frozen development profiles."""
import argparse
import hashlib
import json
import os
import tempfile
from pathlib import Path
import play
from gameplay_bundle import snapshot

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


def configured_definition(package):
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
    return document(path)


def configured_pursuit(package):
    return configured_definition(package).get('aggro') is not None


def profile_role(flag):
    return {'--abilities': 'abilities', '--character-stats': 'stats', '--statuses': 'statuses',
            '--items': 'items', '--reward-catalog': 'rewards', '--progression-catalog': 'progression',
            '--quest-catalog': 'quest', '--dialogue-catalog': 'dialogue', '--vendor-catalog': 'vendor',
            '--document-catalog': 'documents'}.get(flag, flag.removeprefix('--').replace('-', '_'))


def test_route_snapshot(path, package):
    """Freeze an explicitly selected test route; Rust still checks every pose."""
    if path is None:
        return None
    path = path.resolve()
    with path.open('rb') as file:
        source = file.read(32 * 1024 + 1)
    if len(source) > 32 * 1024:
        raise ValueError('Test route exceeds 32 KiB')
    document = json.loads(source)
    if not isinstance(document, dict) or document.get('schema') != 1 or document.get('start_paused') is not True:
        raise ValueError('Account test route must use schema 1 and start paused')
    navigation = document.get('navigation')
    if not isinstance(navigation, str) or not navigation or (path.parent/navigation).resolve() != (package/'navigation').resolve():
        raise ValueError('Test route must use the selected population navigation')
    document['navigation'] = str((package/'navigation').resolve())
    data = (json.dumps(document, indent=2, allow_nan=False)+'\n').encode()
    if len(data) > 32 * 1024:
        raise ValueError('Frozen test route exceeds 32 KiB')
    return data, dict(source_sha256=hashlib.sha256(source).hexdigest(), sha256=hashlib.sha256(data).hexdigest())


def prepare(args, repository):
    manifest = getattr(args, 'gameplay_manifest', None)
    validator = getattr(args, 'gameplay_validator', None)
    if validator and not manifest:
        raise ValueError('Gameplay validator requires a gameplay manifest')
    if manifest:
        if any(getattr(args, key, None) for key in ['items', 'reward_catalog', 'quest_catalog',
                'dialogue_catalog', 'guide_position', 'quest_areas']):
            raise ValueError('Gameplay manifest cannot be mixed with individual profile overrides')
        validator = validator or args.binaries/'examples'/('validate_gameplay.exe' if os.name == 'nt' else 'validate_gameplay')
        with tempfile.TemporaryDirectory(prefix='theia-population-gameplay-') as temporary:
            with snapshot(manifest, validator, Path(temporary)) as bundle:
                if bundle[1]['manifest_schema'] < 16:
                    raise ValueError('Population gameplay snapshots require schema 16 or newer')
                return prepare_profiles(args, repository, bundle)
    return prepare_profiles(args, repository)


def prepare_profiles(args, repository, bundle=None):
    paths, report = bundle if bundle else ({}, None)
    package, output = args.package.resolve(), args.output.resolve()
    players, binaries = args.players.resolve(), args.binaries.resolve()
    for path in [package/'population.json', package/'world/authored.world.json', players/'character.glb']:
        if not path.is_file():
            raise ValueError(f'Missing account playtest dependency: {path}')
    items = paths.get('items', getattr(args, 'items', None))
    rewards = paths.get('rewards', getattr(args, 'reward_catalog', None))
    if bool(items) != bool(rewards):
        raise ValueError('Select item and reward catalogues together')
    quests = paths.get('quest', getattr(args, 'quest_catalog', None))
    dialogue = paths.get('dialogue', getattr(args, 'dialogue_catalog', None))
    guide = 'authored' if bundle else getattr(args, 'guide_position', None)
    areas = paths.get('quest_areas', getattr(args, 'quest_areas', None))
    if any([quests, dialogue, guide]) and not all([quests, dialogue, guide]):
        raise ValueError('Select quest and dialogue catalogues and a guide position together')
    if areas and not quests:
        raise ValueError('Quest areas require an explicit quest journal')
    definition = configured_definition(package)
    pursuit = definition.get('aggro') is not None
    if bundle and json.loads(paths['creature'].read_bytes())['id'] != definition.get('id'):
        raise ValueError('Population creature ID differs from the gameplay snapshot')
    encounter = pursuit or bool(bundle)
    quest_items = bool(quests) and (not report or report['manifest_schema'] >= 18)
    overrides = {'--items': items, '--reward-catalog': rewards}
    selected_names = {'--items': 'selected-items.json', '--reward-catalog': 'selected-rewards.json',
                      '--quest-catalog': 'selected-quests.json', '--dialogue-catalog': 'selected-dialogue.json',
                      '--quest-areas': 'selected-areas.json'}
    quest_profiles = []
    if quests:
        overrides.update({'--quest-catalog': quests, '--dialogue-catalog': dialogue})
        quest_profiles = [(flag, selected_names[flag]) for flag in ['--quest-catalog', '--dialogue-catalog']]
        if areas:
            overrides['--quest-areas'] = areas
            quest_profiles.append(('--quest-areas', selected_names['--quest-areas']))
    if bundle:
        for flag in ['--vendor-catalog', '--document-catalog', '--quest-item-rewards']:
            if profile_role(flag) in paths:
                quest_profiles.append((flag, 'selected-'+profile_role(flag).replace('_', '-')+'.json'))
        for flag, _ in [*PROFILES, RESET_PROFILE, THREAT_PROFILE, *quest_profiles]:
            overrides[flag] = paths[profile_role(flag)]
            selected_names.setdefault(flag, 'selected-'+profile_role(flag).replace('_', '-')+'.json')
    profiles = []
    corpse_loot = False
    quest_loot = False
    for flag, name in [*PROFILES, *([RESET_PROFILE, THREAT_PROFILE] if encounter else []), *quest_profiles]:
        selected = overrides.get(flag)
        source = selected.resolve() if selected else repository/'content/gameplay'/name
        if selected:
            name = selected_names[flag]
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
            quest_loot = any(isinstance(row, dict) and row.get('quest_required') is True
                             for row in document['rewards'])
        profiles.append((flag, name, data))
    if quest_loot and (not quest_items or not corpse_loot):
        raise ValueError('Quest-required loot needs an explicit inventory-bound quest journal and per-life rewards')
    route = test_route_snapshot(getattr(args, 'test_route', None), package)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    frozen = output/'profiles'
    frozen.mkdir(mode=0o700)
    command = ['--authored-world', str(package/'world'), '--population-config', str(package/'population.json'),
               '--players', str(players), '--binaries', str(binaries), '--window-size', args.window_size,
               '--classic', '--account-login', '--equipment', '--status-effects', '--progression', '--rewards',
               '--public-avatar-life', '--stop-on-stdin-eof', '--session-directory', str(output/'runtime')]
    skin = getattr(args, 'fantasy_ui', None)
    if getattr(args, 'parties', False):
        command.append('--parties')
    preferences = getattr(args, 'ui_preferences', None)
    if skin and preferences is None:
        preferences = repository/'build/user-preferences/interface.json'
    command.extend(play.fantasy_ui_options(skin, True, True, ui_preferences=preferences))
    if corpse_loot:
        command.append('--creature-loot')
    if quests:
        command.extend(['--quest-journal', '--guide-position', guide])
    if quest_items:
        command.append('--quest-items')
    if 'vendor' in paths:
        command.append('--vendors')
    if quest_loot:
        command.append('--quest-loot')
    if encounter:
        command.append('--public-creature-attacks')
    command.extend(['--character-id', args.character_id] if args.character_id else ['--character-picker'])
    if route:
        (output/'test-route.json').write_bytes(route[0])
        command.extend(['--test-route', str(output/'test-route.json')])
    if (package/'creature.glb').is_file():
        command.extend(['--creature-asset', str(package/'creature.glb')])
    if getattr(args, 'test_window', None):
        command += ['--test-window', args.test_window]
    records = []
    for flag, name, data in profiles:
        (frozen/name).write_bytes(data)
        command.extend([flag, str(frozen/name)])
        records.append(dict(option=flag, file=f'profiles/{name}', sha256=hashlib.sha256(data).hexdigest()))
    (output/'launch.json').write_text(json.dumps(dict(schema=1, package=str(package), profiles=records, creature_loot=corpse_loot, creature_reset=encounter, creature_threat=encounter, public_creature_attacks=encounter,
        quest_journal=bool(quests), quest_items=quest_items, quest_loot=quest_loot, quest_areas=bool(areas), guide_position=guide, gameplay_snapshot=report,
        test_route=route[1] if route else None, parties=bool(getattr(args, 'parties', False)),
        scope='One authenticated character on the exported population with frozen gameplay profiles and durable grants; shared NPC retaliation and configured pursuit, return reset, damage threat and public attack timing; no Test Player controllers'), indent=2)+'\n')
    return command


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--players', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fantasy-ui', type=Path, default=os.environ.get('THEIA_FANTASY_UI'), help='Local Synty Fantasy Warrior HUD')
    parser.add_argument('--ui-preferences', type=Path, help='Local interface/graphics settings; defaults to build/user-preferences/interface.json with the HUD')
    parser.add_argument('--parties', action='store_true', help='Enable party membership controls in this account test')
    parser.add_argument('--test-window', help='Monitor tile INDEX/COUNT for this account client')
    parser.add_argument('--window-size', default='3440x1440')
    parser.add_argument('--test-route', type=Path, help='Explicit paused test travel; frozen locally and checked by normal client preflight')
    parser.add_argument('--items', type=Path, help='Explicit item catalogue, paired with --reward-catalog')
    parser.add_argument('--reward-catalog', type=Path, help='Explicit rewards; per-life entries enable ordinary corpse pickup')
    parser.add_argument('--quest-catalog', type=Path, help='Explicit inventory-bound expanded journal; paired with dialogue and guide position')
    parser.add_argument('--dialogue-catalog', type=Path, help='Explicit quest contact dialogue; requires --quest-catalog')
    parser.add_argument('--guide-position', help='Explicit quest contact x,z or x,y,z feet, or authored; requires --quest-catalog')
    parser.add_argument('--quest-areas', type=Path, help='Explicit authored quest/map areas; requires --quest-catalog')
    parser.add_argument('--gameplay-manifest', type=Path, help='Complete validated schema-16+ gameplay snapshot; uses the authored guide and excludes individual overrides')
    parser.add_argument('--gameplay-validator', type=Path, help='Optional Rust validator executable for --gameplay-manifest')
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
