"""Generate both sides of Theia's authored public playtest configuration."""
import argparse
import json
from pathlib import Path

from client_install import InstallError, read_json, regular, relative


GAMEPLAY = Path('world/gameplay')
CATALOGS = {
    'combat-catalog': 'abilities',
    'character-stats': 'stats',
    'status-catalog': 'statuses',
    'creature-retaliation': 'creature_retaliation',
    'player-respawn': 'player_respawn',
    'item-catalog': 'items',
    'reward-catalog': 'rewards',
    'progression-catalog': 'progression',
    'quest-catalog': 'quest',
    'quest-areas': 'quest_areas',
    'dialogue-catalog': 'dialogue',
    'vendor-catalog': 'vendor',
    'document-catalog': 'documents',
    'quest-item-rewards': 'quest_item_rewards',
    'melee-weapons': 'melee_weapons',
    'physical-definitions': 'physical_definitions',
    'physical-stats': 'physical_stats',
    'white-melee': 'white_melee',
    'ability-starting': 'ability_starting',
    'ability-damage': 'ability_damage',
    'combat-membership': 'combat_membership',
    'creature-credit': 'creature_credit',
    'creature-reset': 'creature_reset',
    'creature-threat': 'creature_threat',
}
CLIENT_OPTIONS = [
    '--creature-asset', 'creature.glb', '--player-assets', 'players',
    '--zone-map', 'theia/zone-map', '--spell-feedback', 'theia/spell-feedback.json',
]


def arguments(payload, target):
    payload = payload.resolve(strict=True)
    manifest = read_json(payload / GAMEPLAY / 'gameplay.manifest.json', 8192)
    if not isinstance(manifest, dict) or manifest.get('schema') != 21:
        raise InstallError('Theia public playtest requires the complete schema-21 gameplay export')
    shared = [
        '--authored-world', 'world', '--prediction', '--classic',
        '--population-config', 'population.json', '--chat-area', '7',
        '--guide-position', 'authored', '--quest-journal', '--quest-items',
        '--quest-loot', '--creature-loot', '--public-avatar-life',
        '--public-creature-attacks', '--parties',
    ]
    for flag, role in CATALOGS.items():
        path = GAMEPLAY / relative(manifest.get(role))
        regular(payload / path)
        shared.extend([f'--{flag}', path.as_posix()])
    if target == 'client':
        return [*shared, *CLIENT_OPTIONS]
    return shared


def verify_report(report):
    population = report['population'] or {}
    if not (report['interface']['enabled'] and report['zone_map'] and report['spell_feedback']
            and report['contacts'] > 0 and population.get('count', 0) > 0
            and report['creature_appearance'] and report['creature_loot']
            and report['ability_count'] > 1 and all(report['typed_combat'].values())
            and report['inventory_profile'] and report['journal_profile']
            and report['progression_profile'] and report['fixed_reward_catalog']
            and report['chat_area'] and report['parties']
            and not (report['renderer_started'] or report['network_started'])):
        raise InstallError('The exported client does not enable the full Theia playtest')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('target', choices=['client', 'server'])
    parser.add_argument('payload', type=Path)
    args = parser.parse_args()
    print(json.dumps(arguments(args.payload, args.target)))
