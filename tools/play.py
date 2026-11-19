#!/usr/bin/env python3
"""Launch the continental graybox or an explicit legacy map, with child cleanup."""
import argparse
from contextlib import ExitStack
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import tempfile
from launch_environment import client_environment


def play(world, players, clients, binaries, tick_hz=30, transport="renet", chunk_world=None, camera_collision=None, graybox=False, dev_speed=1, dev_spawn="8000,8000", dev_tour=False, prediction=False, saved_terrain=None, targeting=False, combat_catalog=None, feedback=None, persistence=False, account_login=False, character_stats=None, item_catalog=None, status_catalog=None, creature_profile=None, navigation=None, reward_catalog=None, progression_catalog=None, window_size=None, fullscreen=False, quest_catalog=None, dialogue_catalog=None, guide_position=None, vendor_catalog=None, quest_lifecycle=False, quest_turn_in_xp=None, creature_retaliation=None, player_respawn=None, creature_asset=None, dodge_profile=None, classic=False, controls_file=None, melee_weapons=None, physical_definitions=None, physical_stats=None, white_melee=None, ability_starting=None, ability_damage=None, combat_membership=None, creature_credit=None, creature_reset=None, quest_journal=False, creature_threat=None, public_creature_attacks=False, public_avatar_life=False, quest_areas=None):
    window_size = window_size or os.environ.get('THEIA_WINDOW_SIZE', '1920x1200')
    match = re.fullmatch(r'(\d+)x(\d+)', window_size)
    if not match or not (640 <= int(match[1]) <= 7680 and 480 <= int(match[2]) <= 4320):
        raise ValueError('Expected --window-size WIDTHxHEIGHT within 640x480 to 7680x4320')
    if classic and (not prediction or not graybox or transport != 'renet' or dev_tour or dev_speed != 1 or dodge_profile):
        raise ValueError('Classic controls require exclusive normal-speed predicted secure Renet movement')
    if melee_weapons and (not classic or not combat_catalog or not item_catalog or not account_login):
        raise ValueError('Melee weapons require classic controls, combat and account-backed equipment')
    if bool(physical_definitions) != bool(physical_stats):
        raise ValueError('Physical damage definitions and stats must be selected together')
    if physical_definitions and (not melee_weapons or not creature_retaliation or not player_respawn):
        raise ValueError('Physical damage requires classic melee and player life profiles')
    if white_melee and not physical_definitions:
        raise ValueError('Ordinary outcomes require physical damage profiles')
    if bool(ability_starting) != bool(ability_damage):
        raise ValueError('Ability starting and damage policy must be selected together')
    if ability_damage and not white_melee:
        raise ValueError('Ability damage requires ordinary physical combat profiles')
    if combat_membership and (not ability_damage or not classic or not creature_retaliation or not player_respawn):
        raise ValueError('Combat membership requires classic ability damage and player life')
    if public_avatar_life and (not classic or not creature_retaliation or not player_respawn):
        raise ValueError("Public avatar life requires Classic player life")
    if public_creature_attacks and (not classic or not creature_retaliation or not player_respawn):
        raise ValueError("Public creature attacks require Classic player life")
    if creature_threat and not creature_reset:
        raise ValueError("Creature threat requires explicit return reset")
    if creature_reset and not creature_credit:
        raise ValueError("Creature reset requires explicit solo credit")
    if creature_credit and not combat_membership:
        raise ValueError("Creature credit requires explicit combat membership")
    if controls_file and not classic:
        raise ValueError('Controls file requires classic controls')
    if dodge_profile and (not prediction or not graybox or transport != 'renet' or dev_tour or dev_speed != 1):
        raise ValueError('Dodge requires normal-speed predicted terrain and secure Renet without a tour')
    if bool(creature_retaliation) != bool(player_respawn):
        raise ValueError('Retaliation and player respawn profiles must be selected together')
    if creature_retaliation and (not prediction or not creature_profile or not character_stats or not combat_catalog):
        raise ValueError('Player life requires predicted creature combat and character stats')
    if quest_journal and (quest_lifecycle or quest_turn_in_xp or not quest_catalog or not dialogue_catalog or not account_login or not prediction):
        raise ValueError('Journal requires exclusive predicted quest mode, catalogue, dialogue and account login')
    if quest_turn_in_xp and (not quest_lifecycle or not progression_catalog):
        raise ValueError('Completion XP requires explicit quests and progression')
    if quest_lifecycle and (not quest_catalog or not dialogue_catalog or not account_login):
        raise ValueError('Explicit quests require quest profile, dialogue and account login')
    if vendor_catalog and (not dialogue_catalog or not account_login or not item_catalog):
        raise ValueError('Vendor requires guide dialogue, account login and items')
    if bool(dialogue_catalog) != bool(guide_position):
        raise ValueError('Dialogue requires a catalog and explicit guide position')
    if dialogue_catalog:
        if not graybox or not prediction or transport != 'renet':
            raise ValueError('Dialogue requires predicted graybox and secure Renet')
        guide_values=[float(value) for value in guide_position.split(',')]
        if len(guide_values)!=2 or not all(math.isfinite(value) and 0 <= value < 16000 for value in guide_values):
            raise ValueError('Guide position must be finite graybox x,z metres')
    if bool(creature_profile) != bool(navigation):
        raise ValueError('Creature profile and navigation package must be selected together')
    if creature_profile and (not combat_catalog or not prediction or saved_terrain is None):
        raise ValueError('Creature patrol requires predicted saved-terrain combat')
    if status_catalog and (not character_stats or not combat_catalog):
        raise ValueError('Statuses require combat and character stats')
    if quest_catalog and not progression_catalog:
        raise ValueError("Quest objectives require progression")
    if progression_catalog and (not account_login or not creature_profile):
        raise ValueError("Progression requires account login and creatures")
    if reward_catalog and (not account_login or not item_catalog or not creature_profile):
        raise ValueError('Rewards require account login, items and creature combat')
    if item_catalog and not account_login:
        raise ValueError('Persistent inventory requires --account-login')
    if character_stats and not combat_catalog:
        raise ValueError('Character stats require combat')
    if account_login:
        if persistence or saved_terrain is None or not prediction or transport != 'renet' or clients != 1 or dev_tour or dev_speed != 1:
            raise ValueError('Account login requires one client, normal predicted saved terrain and Renet; excludes development persistence/tour')
        if any(not os.environ.get(key) for key in ['THEIA_DATABASE_URL','THEIA_OIDC_ISSUER','THEIA_OIDC_CLIENT_ID']):
            raise ValueError('Account login requires THEIA_DATABASE_URL, THEIA_OIDC_ISSUER and THEIA_OIDC_CLIENT_ID; see docs/AUTHENTICATED_ADMISSION.md')
    if persistence and (saved_terrain is None or not prediction or transport != 'renet' or dev_tour or dev_speed != 1):
        raise ValueError('Persistence requires saved terrain, normal-speed prediction and Renet without a tour')
    if persistence and not os.environ.get('THEIA_DATABASE_URL'):
        raise ValueError('Persistence requires a local PostgreSQL THEIA_DATABASE_URL; see docs/PERSISTENCE_FOUNDATION.md')
    if saved_terrain is not None:
        if world is not None or chunk_world is not None:
            raise ValueError('Choose saved terrain or an imported world')
        if not (saved_terrain / 'terrain.manifest.json').is_file():
            raise ValueError(f'Missing saved terrain at {saved_terrain}; bake once with cargo run --release --locked -p theia-network --example bake_terrain -- bake "{saved_terrain}"; see docs/SAVED_TERRAIN.md')
        graybox = True
    if creature_profile and not (navigation / 'navigation.manifest.json').is_file():
        raise ValueError(f'Missing baked navigation at {navigation}; run cargo run --release --locked -p theia-nav-bake -- bake "{saved_terrain}" "{navigation}" first')
    if feedback and not combat_catalog:
        raise ValueError("Spell feedback requires combat")
    if feedback:
        required_feedback = feedback
        if not required_feedback.is_file():
            raise ValueError(f"Missing spell feedback {feedback}")
    if combat_catalog is not None:
        targeting = True
    if targeting and (not graybox or transport != 'renet'):
        raise ValueError("Targeting requires the graybox world and authenticated Renet")
    if prediction and not graybox:
        raise ValueError("Prediction requires the graybox world")
    if not graybox and (dev_speed != 1 or dev_spawn != "8000,8000" or dev_tour):
        raise ValueError('Developer travel options require the graybox world')
    if (chunk_world or graybox) and transport != 'renet':
        raise ValueError('Chunk worlds require authenticated Renet readiness')
    suffix = '.exe' if os.name == 'nt' else ''
    server_binary = binaries / ('theia-server' + suffix)
    client_binary = binaries / ('theia-client' + suffix)
    required = ([saved_terrain / 'terrain.manifest.json'] if saved_terrain else []) if graybox else ([chunk_world / 'world.chunks.json', chunk_world / 'world.session.json'] if chunk_world else
                [world / 'world.manifest.json', world / 'terrain.json', world / 'terrain.glb'])
    scene = [*(['--saved-terrain', str(saved_terrain)] if saved_terrain else ['--graybox-world']), *(['--prediction'] if prediction else []), *(['--targeting'] if targeting else [])] if graybox else (['--chunk-world', str(chunk_world)] if chunk_world else ['--terrain-world', str(world)])
    if classic:scene.append('--classic')
    if vendor_catalog:
        required.append(vendor_catalog)
        scene.extend(['--vendor-catalog',str(vendor_catalog)])
    if dialogue_catalog:
        required.append(dialogue_catalog)
        scene.extend(['--dialogue-catalog',str(dialogue_catalog),'--guide-position',guide_position])
    if combat_catalog is not None:
        required.append(combat_catalog)
        scene.extend(['--combat-catalog', str(combat_catalog)])
    if character_stats is not None:
        required.append(character_stats)
        scene.extend(['--character-stats', str(character_stats)])
    if status_catalog is not None:
        required.append(status_catalog)
        scene.extend(['--status-catalog', str(status_catalog)])
    if creature_profile is not None:
        required.extend([creature_profile, navigation / 'navigation.manifest.json'])
        scene.extend(['--creature-profile', str(creature_profile), '--navigation', str(navigation)])
    if creature_retaliation is not None:
        required.extend([creature_retaliation, player_respawn])
        scene.extend(['--creature-retaliation', str(creature_retaliation), '--player-respawn', str(player_respawn)])
    if white_melee is not None:
        required.append(white_melee)
        scene.extend(['--white-melee', str(white_melee)])
    if public_avatar_life:
        scene.append("--public-avatar-life")
    if public_creature_attacks:
        scene.append("--public-creature-attacks")
    if creature_threat is not None:
        required.append(creature_threat)
        scene.extend(["--creature-threat", str(creature_threat)])
    if creature_reset is not None:
        required.append(creature_reset)
        scene.extend(['--creature-reset', str(creature_reset)])
    if creature_credit is not None:
        required.append(creature_credit)
        scene.extend(['--creature-credit', str(creature_credit)])
    if combat_membership is not None:
        required.append(combat_membership)
        scene.extend(['--combat-membership', str(combat_membership)])
    if ability_starting is not None:
        required.extend([ability_starting, ability_damage])
        scene.extend(['--ability-starting', str(ability_starting), '--ability-damage', str(ability_damage)])
    if physical_definitions is not None:
        required.extend([physical_definitions, physical_stats])
        scene.extend(['--physical-definitions', str(physical_definitions), '--physical-stats', str(physical_stats)])
    if melee_weapons is not None:
        required.append(melee_weapons)
        scene.extend(['--melee-weapons', str(melee_weapons)])
    if dodge_profile is not None:
        required.append(dodge_profile)
        scene.extend(['--dodge-profile', str(dodge_profile)])
    if item_catalog is not None:
        required.append(item_catalog)
        scene.extend(['--item-catalog', str(item_catalog)])
    if quest_journal:
        scene.append('--quest-journal')
    if quest_lifecycle:
        scene.append('--quest-lifecycle')
    if quest_turn_in_xp is not None:
        required.append(quest_turn_in_xp)
        scene.extend(['--quest-turn-in-xp', str(quest_turn_in_xp)])
    if quest_areas is not None:
        if not quest_journal or quest_catalog is None:
            raise ValueError('Quest areas require expanded journal mode and catalogue')
        required.append(quest_areas)
        scene.extend(['--quest-areas', str(quest_areas)])
    if quest_catalog is not None:
        required.append(quest_catalog)
        scene.extend(['--quest-catalog', str(quest_catalog)])
    if progression_catalog is not None:
        required.append(progression_catalog)
        scene.extend(['--progression-catalog', str(progression_catalog)])
    if reward_catalog is not None:
        required.append(reward_catalog)
        scene.extend(['--reward-catalog', str(reward_catalog)])
    if creature_asset is not None:
        if not creature_profile:
            raise ValueError('Creature art requires authoritative creature combat')
        if not creature_asset.is_file():
            raise ValueError(f'Missing creature art: {creature_asset}')
    server_options = ['--dev-speed', str(dev_speed), '--dev-spawn', dev_spawn, *(['--dev-tour'] if dev_tour else [])] if graybox else []
    for path in [*required, players / ('character.glb' if graybox else 'human.glb'), server_binary, client_binary]:
        if not path.is_file():
            raise ValueError(f'Missing {path}; build with cargo build --release --locked --workspace; see docs/DEVELOPMENT_WORLD.md for asset preparation or select --binaries explicitly')
    session = Path('build/development-session')
    session.mkdir(parents=True, exist_ok=True)
    certificate = session / 'server-cert.der'
    certificate.unlink(missing_ok=True)
    tickets = tempfile.TemporaryDirectory(prefix="renet-", dir=session) if transport == "renet" else None
    processes = []
    account = None
    try:
        with (session / 'server.log').open('w') as log:
            server = subprocess.Popen([str(server_binary), '127.0.0.1:0', str(certificate),
                                       *scene, *server_options, *(['--dev-persistence'] if persistence else []), '--tick-hz', str(tick_hz),
                                       *(['--account-login'] if account_login else (['--renet-tickets', tickets.name] if tickets else []))],
                                      stdout=log, stderr=subprocess.STDOUT)
            processes.append(server)
            address = None
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if server.poll() is not None:
                    raise RuntimeError(f'Server failed; see {session / "server.log"}')
                if account_login:
                    ready = next((json.loads(line.removeprefix('account_game_ready ')) for line in (session/'server.log').read_text().splitlines() if line.startswith('account_game_ready ')),None)
                    if ready:
                        from account_login import AccountClient
                        account = AccountClient(ready['account_address'],alive=lambda:server.poll() is None)
                        print('Sign in using your system browser, then return here to choose your character.',flush=True)
                        account.login()
                        address = account.choose_and_enter(Path(tickets.name)/'client-1.token')
                        break
                match = re.search(r'listening=(\S+) (?:public_certificate|local_tickets)=', (session / 'server.log').read_text())
                if match and (tickets or certificate.exists()):
                    address = match[1]
                    break
                time.sleep(.05)
            if address is None:
                raise RuntimeError('Server readiness timed out')
            for index in range(clients):
                processes.append(subprocess.Popen([str(client_binary), '--window-size', window_size, *(['--fullscreen'] if fullscreen else []),
                    *(['--controls-file',str(controls_file.resolve())] if controls_file else []), *(['--camera-collision', camera_collision] if camera_collision else []), *(['--spell-feedback', str(feedback)] if feedback else []), *(['--creature-asset', str(creature_asset.resolve())] if creature_asset else []), *scene,
                    '--player-assets', str(players),
                    '--renet-connect' if tickets else '--connect', address,
                    str(Path(tickets.name) / f'client-{index+1}.token') if tickets else str(certificate)],
                    env=client_environment(os.environ)))
            if combat_catalog:
                print('Combat: click the action slot or press 1 to cast; health, cooldown and respawn come from the server.', flush=True)
            if targeting:
                print('Training target: Tab selects/cycles; Backspace clears.', flush=True)
            if classic:
                print('Theia Classic: character-relative movement; walk/run toggle from controls settings (default NumpadDivide); LMB orbit, RMB steer, wheel zoom, Escape closes panels or clears target.', flush=True)
            else:
                print('Theia development map: WASD, Shift to run, Space to jump, RMB orbit, wheel zoom, Escape to quit.', flush=True)
            while all(p.poll() is None for p in processes):
                time.sleep(.2)
            if any(p.poll() not in (None, 0) for p in processes):
                raise RuntimeError('A game process failed; inspect its output and build/development-session/server.log')
    except KeyboardInterrupt:
        pass
    finally:
        if account:
            account.close()
        for process in reversed(processes):
            if process.poll() is None:
                # Let the POSIX server finish its simulation/transport summary.
                # Windows retains terminate until native console shutdown is qualified.
                if process is processes[0] and os.name != 'nt':
                    process.send_signal(signal.SIGINT)
                else:
                    process.terminate()
                try:
                    process.wait(timeout=15 if process is processes[0] and (persistence or account_login) else 5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()

        if tickets:
            tickets.cleanup()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    worlds = parser.add_mutually_exclusive_group()
    worlds.add_argument('--world', type=Path, help='Explicit legacy monolithic world')
    worlds.add_argument('--graybox-world', action='store_true', help='Retained generated terrain baseline')
    worlds.add_argument('--saved-terrain', type=Path, help='Versioned saved height terrain directory')
    worlds.add_argument('--chunk-world', type=Path, help='Opt-in prepared chunk index/session; requires Renet')
    parser.add_argument('--gameplay-manifest', type=Path, help='Validated gameplay bundle; schema 2 includes explicit completion XP')
    parser.add_argument('--gameplay-validator', type=Path, help='Rust validate_gameplay executable; default <binaries>/validate_gameplay')
    parser.add_argument('--fullscreen', action='store_true', help='Borderless fullscreen on the current monitor; F11 toggles in game')
    parser.add_argument('--window-size', help='Client window WIDTHxHEIGHT (default 1920x1200, or THEIA_WINDOW_SIZE)')
    parser.add_argument('--players', type=Path, help='Player assets; defaults to POLYGON for graybox')
    parser.add_argument('--prediction', action=argparse.BooleanOptionalAction, default=None, help='Movement prediction (default for graybox; --no-prediction retains server-pose interpolation)')
    parser.add_argument('--account-login', action='store_true', help='System-browser OIDC and owned character entry; requires configured loopback provider/database')
    parser.add_argument('--persistence', action='store_true', help='Opt-in trusted development character checkpoints; requires local THEIA_DATABASE_URL')
    parser.add_argument('--spell-feedback', action='store_true', help='Opt-in original spell pulses and spatial audio; implies combat')
    parser.add_argument('--combat', action='store_true', help='Opt-in authoritative mobile ability, health, death and respawn; implies targeting')
    parser.add_argument('--inventory', action='store_true', help='Opt-in persistent development backpack; requires --account-login')
    parser.add_argument('--vendors',action='store_true',help='Opt-in guide barter; implies dialogue/equipment, requires account login')
    parser.add_argument('--vendor-catalog',type=Path,help='Barter offers; requires --vendors')
    parser.add_argument('--dialogue',action='store_true',help='Opt-in noncombat informational guide; E to talk, F1–F4 or buttons to choose, Esc to close')
    parser.add_argument('--dialogue-catalog',type=Path,help='Informational dialogue profile; requires --dialogue')
    parser.add_argument('--guide-position',help='Fixed guide x,z metres; default 8004,8000, independent of player spawn')
    parser.add_argument('--quest-turn-in-xp', type=Path, help='Optional completion XP recipe; implies explicit quest/account setup')
    parser.add_argument('--quest-areas', type=Path, help='Explicit static area catalogue for expanded quests')
    parser.add_argument('--quest-journal', action='store_true', help='Expanded multi-quest journal; implies quests/dialogue/account login')
    parser.add_argument('--quest-lifecycle', action='store_true', help='Explicit guide acceptance/turn-in; defaults to journal-only, implies quests/dialogue/account login')
    parser.add_argument('--quests', action='store_true', help='Opt-in development kill objective; implies progression/creatures, requires account login')
    parser.add_argument('--progression', action='store_true', help='Opt-in durable kill XP; implies creatures, requires account login')
    parser.add_argument('--rewards', action='store_true', help='Opt-in first training-kill reward; implies equipment and creatures, requires account login')
    parser.add_argument('--equipment', action='store_true', help='Opt-in development main-hand equipment; implies inventory and requires --account-login')
    parser.add_argument('--items', type=Path, help='Development item catalog; requires --inventory')
    parser.add_argument('--creatures', action='store_true', help='Opt-in server patrol on baked navigation; implies resources/combat')
    parser.add_argument('--creature-profile', type=Path, help='Patrol definition; requires --creatures')
    parser.add_argument('--navigation', type=Path, help='Offline navigation package; requires --creatures')
    parser.add_argument('--status-effects', action='store_true', help='Opt-in authoritative Scorch periodic damage; implies resources and combat')
    parser.add_argument('--statuses', type=Path, help='Status definitions; requires --status-effects')
    parser.add_argument('--resources', action='store_true', help='Opt-in transient development health/mana and cast costs; implies combat')
    parser.add_argument('--creature-retaliation', type=Path, help='Opt-in checked melee retaliation; requires --player-respawn')
    parser.add_argument('--player-respawn', type=Path, help='Transient player-life respawn profile; requires --creature-retaliation')
    parser.add_argument('--creature-asset', type=Path, help='Opt-in converted wolf GLB; requires creature combat')
    parser.add_argument('--character-stats', type=Path, help='Development vitals profile; requires --resources')
    parser.add_argument('--abilities', type=Path, help='Combat catalog (default: content/gameplay/development-abilities.json); requires --combat')
    parser.add_argument('--targeting', action='store_true', help='Opt-in authoritative training target, Tab selection and Backspace clear')
    parser.add_argument('--dev-speed', type=int, choices=[1, 8, 32], default=1)
    parser.add_argument('--dev-spawn', default='8000,8000', help='Graybox server spawn x,z in metres')
    parser.add_argument('--dev-tour', action='store_true', help='Server-owned teleport circuit for streaming qualification')
    parser.add_argument('--clients', type=int, choices=range(1, 9), default=1)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'),
                        help='Optimized interactive binaries; use target/debug explicitly for debugging')
    parser.add_argument('--tick-hz', type=int, choices=[20, 30, 60], default=30)
    parser.add_argument('--transport', choices=['quinn', 'renet'], default='renet')
    parser.add_argument('--camera-collision', choices=['world', 'visual'],
                        help='Override authored world camera clearance or legacy visual-mesh refinement')
    parser.add_argument('--classic',action='store_true',help='Strict classic movement/facing profile; excludes dodge/action camera')
    parser.add_argument('--controls-file',type=Path,help='Classic movement key bindings JSON')
    parser.add_argument('--white-melee', type=Path, help='Explicit ordinary-melee outcomes; requires physical damage and pursuit')
    parser.add_argument('--ability-starting', type=Path, help='Explicit paired ability starting recipes')
    parser.add_argument('--public-avatar-life', action='store_true', help='Public remote death/fresh-life presentation; requires Classic player life')
    parser.add_argument('--public-creature-attacks', action='store_true', help='Explicit public NPC attack timing; requires Classic player life')
    parser.add_argument('--creature-threat', type=Path, help='Explicit damage-threat victim switching; requires return reset')
    parser.add_argument('--creature-reset', type=Path, help='Explicit return health/effect reset; requires solo credit')
    parser.add_argument('--creature-credit', type=Path, help='Explicit solo tag-owner credit; requires combat membership')
    parser.add_argument('--combat-membership', type=Path, help='Owner-only combat state policy; requires classic ability damage and player life')
    parser.add_argument('--ability-damage', type=Path, help='Explicit ability outcome/damage policy; requires ordinary combat')
    parser.add_argument('--physical-definitions', type=Path, help='Explicit physical damage definitions; requires physical stats and classic melee')
    parser.add_argument('--physical-stats', type=Path, help='Explicit physical damage stat templates')
    parser.add_argument('--melee-weapons', type=Path, help='Explicit weapon-timed melee profile; requires classic account-backed combat')
    parser.add_argument('--dodge-profile', type=Path, help='Opt-in checked dodge movement; V key')
    args = parser.parse_args(argv)
    if args.gameplay_validator and not args.gameplay_manifest:
        parser.error('--gameplay-validator requires --gameplay-manifest')
    args.quest_lifecycle = args.quest_lifecycle or args.quest_turn_in_xp is not None
    args.quests = args.quests or args.quest_lifecycle or args.quest_journal
    args.dialogue = args.dialogue or args.quest_lifecycle or args.quest_journal
    args.account_login = args.account_login or args.quest_lifecycle or args.quest_journal
    with ExitStack() as stack:
        bundle = None
        if args.gameplay_manifest:
            if any([args.world, args.chunk_world, args.graybox_world, args.prediction is False, args.transport != 'renet', args.persistence, args.dev_tour, args.dev_speed != 1, args.clients != 1]):
                parser.error('--gameplay-manifest requires one normal predicted saved-terrain Renet client')
            if any([args.items, args.abilities, args.character_stats, args.statuses, args.creature_profile, args.dialogue_catalog, args.vendor_catalog, args.quest_turn_in_xp, args.creature_retaliation, args.player_respawn, args.dodge_profile, args.melee_weapons, args.physical_definitions, args.physical_stats, args.white_melee, args.ability_starting, args.ability_damage, args.combat_membership, args.creature_credit, args.creature_reset, args.creature_threat, args.quest_areas]):
                parser.error('--gameplay-manifest cannot be mixed with individual gameplay profile overrides')
            from gameplay_bundle import snapshot
            validator = args.gameplay_validator or args.binaries / ('validate_gameplay.exe' if os.name == 'nt' else 'validate_gameplay')
            paths, bundle = stack.enter_context(snapshot(args.gameplay_manifest, validator, Path('build/development-session')))
            if args.public_creature_attacks and bundle['manifest_schema'] < 14:
                parser.error('Public creature attack override requires a schema-14 bundle')
            args.public_creature_attacks = bundle['manifest_schema'] >= 14
            if args.public_avatar_life and bundle['manifest_schema'] < 15:
                parser.error('Public avatar override requires a schema-15 or newer bundle')
            args.public_avatar_life = bundle['manifest_schema'] >= 15
            if bundle['manifest_schema'] >= 12:
                if args.quest_lifecycle or args.quest_turn_in_xp:
                    parser.error('Expanded journal bundle excludes legacy quest overrides')
                args.quest_journal = True
            elif args.quest_journal:
                parser.error('--quest-journal requires a schema-12 or newer gameplay bundle')
            args.account_login = args.inventory = args.equipment = args.rewards = True
            args.quests = args.progression = args.creatures = args.resources = True
            args.status_effects = args.combat = args.dialogue = args.vendors = True
            args.items, args.abilities, args.character_stats = paths['items'], paths['abilities'], paths['stats']
            args.statuses, args.creature_profile = paths['statuses'], paths['creature']
            args.dialogue_catalog, args.vendor_catalog = paths['dialogue'], paths['vendor']
            args.creature_retaliation, args.player_respawn = paths.get('creature_retaliation'), paths.get('player_respawn')
            args.quest_turn_in_xp = paths.get('quest_turn_in_xp')
            args.dodge_profile = paths.get('dodge')
            args.melee_weapons = paths.get('melee_weapons')
            args.physical_definitions = paths.get('physical_definitions')
            args.physical_stats = paths.get('physical_stats')
            args.white_melee = paths.get('white_melee')
            args.ability_starting = paths.get('ability_starting')
            args.ability_damage = paths.get('ability_damage')
            args.combat_membership = paths.get('combat_membership')
            args.creature_credit = paths.get('creature_credit')
            args.creature_reset = paths.get('creature_reset')
            args.creature_threat = paths.get('creature_threat')
            args.quest_areas = paths.get('quest_areas')
            args.classic = args.classic or args.melee_weapons is not None
            args.quest_lifecycle = args.quest_lifecycle or args.quest_turn_in_xp is not None

        if args.vendor_catalog and not args.vendors:
            parser.error('--vendor-catalog requires --vendors')
        args.dialogue = args.dialogue or args.vendors
        vendor_catalog = (args.vendor_catalog or Path('content/gameplay/development-vendor.json')).resolve() if args.vendors else None
        if (args.dialogue_catalog or args.guide_position) and not args.dialogue:
            parser.error('--dialogue-catalog/--guide-position require --dialogue')
        dialogue_catalog=(args.dialogue_catalog or Path('content/quests/development-journal-dialogue.json' if args.quest_journal else 'content/gameplay/development-dialogue.json')).resolve() if args.dialogue else None
        guide_position=(args.guide_position or '8004,8000') if args.dialogue else None
        args.progression = args.progression or args.quests
        quest_catalog = Path('content/quests/development-journal.json' if args.quest_journal else 'content/gameplay/development-quest.json').resolve() if args.quests else None
        args.equipment = args.equipment or args.rewards or args.vendors
        args.creatures = args.creatures or args.rewards or args.progression or args.creature_retaliation is not None
        progression_catalog = Path('content/gameplay/development-progression.json').resolve() if args.progression else None
        reward_catalog = Path('content/gameplay/development-rewards.json').resolve() if args.rewards else None
        args.inventory = args.inventory or args.equipment
        if args.inventory and not args.account_login:
            parser.error('--inventory requires --account-login')
        if args.items and not args.inventory:
            parser.error('--items requires --inventory')
        item_catalog = (args.items or Path('content/gameplay/development-equipment-items.json' if args.equipment else 'content/gameplay/development-items.json')).resolve() if args.inventory else None
        if args.equipment and json.loads(item_catalog.read_text()).get('schema') != 2:
            parser.error('--equipment requires an equipment-capable item catalog')
        if args.statuses and not args.status_effects:
            parser.error('--statuses requires --status-effects')
        args.resources = args.resources or args.status_effects or args.creatures
        args.combat = args.combat or args.spell_feedback or args.resources
        if args.character_stats and not args.resources:
            parser.error('--character-stats requires --resources')
        if args.abilities and not args.combat:
            parser.error('--abilities requires --combat')
        default_catalog = 'development-status-abilities.json' if args.status_effects else ('development-resource-abilities.json' if args.resources else 'development-abilities.json')
        status_catalog = (args.statuses or Path('content/gameplay/development-statuses.json')).resolve() if args.status_effects else None
        feedback = Path('content/gameplay/development-status-feedback.json' if args.status_effects else 'content/gameplay/development-spell-feedback.json').resolve() if args.spell_feedback else None
        combat_catalog = (args.abilities or Path('content/gameplay') / default_catalog).resolve() if args.combat else None
        character_stats = (args.character_stats or Path('content/gameplay/development-character-stats.json')).resolve() if args.resources else None
        if (args.creature_profile or args.navigation) and not args.creatures:
            parser.error('--creature-profile/--navigation require --creatures')
        creature_profile = (args.creature_profile or Path('content/gameplay/development-creature-patrol.json')).resolve() if args.creatures else None
        navigation = (args.navigation or Path('build/navigation-terrain-v1')).resolve() if args.creatures else None
        if bundle:
            quest_catalog, progression_catalog, reward_catalog = paths['quest'], paths['progression'], paths['rewards']
            print(f"Validated gameplay: {bundle['id']} revision {bundle['revision']} ({bundle['validation_bundle_sha256']})", flush=True)
        graybox = args.graybox_world or args.saved_terrain is not None or (args.world is None and args.chunk_world is None)
        saved_terrain = (args.saved_terrain or Path('build/graybox-terrain-v1')).resolve() if graybox and not args.graybox_world else None
        prediction = graybox if args.prediction is None else args.prediction
        players = args.players or Path('build/synty-character-canonical-stage' if graybox else 'build/reference-scene')
        play(args.world.resolve() if args.world else None, players.resolve(), args.clients, args.binaries.resolve(), args.tick_hz, args.transport,
             args.chunk_world.resolve() if args.chunk_world else None, args.camera_collision, graybox, args.dev_speed, args.dev_spawn, args.dev_tour, prediction, saved_terrain, args.targeting, combat_catalog, feedback, args.persistence, args.account_login, character_stats, item_catalog, status_catalog, creature_profile, navigation, reward_catalog, progression_catalog, args.window_size, args.fullscreen, quest_catalog, dialogue_catalog, guide_position, vendor_catalog, args.quest_lifecycle, args.quest_turn_in_xp.resolve() if args.quest_turn_in_xp else None, args.creature_retaliation.resolve() if args.creature_retaliation else None, args.player_respawn.resolve() if args.player_respawn else None, args.creature_asset.resolve() if args.creature_asset else None, args.dodge_profile.resolve() if args.dodge_profile else None, args.classic, args.controls_file, args.melee_weapons.resolve() if args.melee_weapons else None, args.physical_definitions.resolve() if args.physical_definitions else None, args.physical_stats.resolve() if args.physical_stats else None, args.white_melee.resolve() if args.white_melee else None, args.ability_starting.resolve() if args.ability_starting else None, args.ability_damage.resolve() if args.ability_damage else None, args.combat_membership.resolve() if args.combat_membership else None, args.creature_credit.resolve() if args.creature_credit else None, args.creature_reset.resolve() if args.creature_reset else None, args.quest_journal, args.creature_threat.resolve() if args.creature_threat else None, args.public_creature_attacks, args.public_avatar_life, args.quest_areas.resolve() if args.quest_areas else None)


if __name__ == '__main__':
    main()
