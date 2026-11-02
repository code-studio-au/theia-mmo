"""Exact-byte launcher handoff using the actual Rust validator; no native proof."""
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import inspect
import io
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import gameplay_bundle as bundle
import play

PLAY_SIGNATURE = inspect.signature(play.play)


class GameplayBundle(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validator = Path(os.environ.get('THEIA_TEST_GAMEPLAY_VALIDATOR',
                             'target/debug/examples/validate_gameplay')).resolve()
        if not cls.validator.is_file():
            raise unittest.SkipTest('Build the actual Rust validate_gameplay example first')

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'content'
        self.source.mkdir()
        self.manifest = self.source / 'development-gameplay.json'
        original = Path('content/gameplay/development-gameplay.json')
        shutil.copy2(original, self.manifest)
        raw = json.loads(original.read_text())
        for role in bundle.ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        self.session = self.root / 'session'

    def test_snapshot_retains_validated_bytes_after_author_edits_and_cleans_on_exit(self):
        before = {p.name: p.read_bytes() for p in self.source.iterdir()}
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.ROLES)
            self.assertEqual({p.name: p.read_bytes() for p in self.source.iterdir()}, before)
            for r in report['profiles']:
                self.assertEqual(hashlib.sha256(paths[r['kind']].read_bytes()).hexdigest(), r['sha256'])
            copied = paths['quest'].read_bytes()
            (self.source / paths['quest'].name).write_bytes(b'broken authoring edit')
            self.assertEqual(paths['quest'].read_bytes(), copied)
            self.assertEqual(bundle._validate(self.validator, paths['quest'].parent / self.manifest.name), report)
        self.assertFalse(any(self.session.iterdir()))

    def test_edit_between_validation_and_copy_refuses_and_removes_partial_snapshot(self):
        real = bundle._validate
        def validate(*args):
            report = real(*args)
            raw = json.loads(self.manifest.read_text())
            target = self.source / raw['vendor']
            target.write_bytes(target.read_bytes() + b' ')
            return report
        with patch.object(bundle, '_validate', side_effect=validate):
            with self.assertRaisesRegex(bundle.BundleError, 'changed during validation'):
                with bundle.snapshot(self.manifest, self.validator, self.session):
                    self.fail('Changed profile reached launch')
        self.assertFalse(any(self.session.iterdir()))

    def test_invalid_profile_is_refused_before_any_snapshot_is_published(self):
        raw = json.loads(self.manifest.read_text())
        (self.source / raw['vendor']).write_text('{"DO-NOT-ECHO":"private input"}')
        with self.assertRaises(bundle.BundleError) as error:
            with bundle.snapshot(self.manifest, self.validator, self.session):
                self.fail('Invalid profile reached launch')
        self.assertNotIn('DO-NOT-ECHO', str(error.exception))
        self.assertFalse(self.session.exists())

    def test_native_startup_exception_keeps_its_identity_and_cleans_snapshot(self):
        with self.assertRaisesRegex(OSError, 'native child startup'):
            with bundle.snapshot(self.manifest, self.validator, self.session):
                raise OSError('native child startup')
        self.assertFalse(any(self.session.iterdir()))

    def test_launcher_selects_all_ten_snapshot_profiles_and_retains_them_until_play_returns(self):
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            for key in ['combat_catalog', 'character_stats', 'item_catalog', 'status_catalog',
                        'creature_profile', 'reward_catalog', 'progression_catalog',
                        'quest_catalog', 'dialogue_catalog', 'vendor_catalog']:
                path = observed[key]
                self.assertTrue(path.is_file())
                self.assertNotEqual(path.parent, self.source)
            self.assertTrue(observed['account_login'])
            self.assertTrue(observed['prediction'])
            self.assertEqual(observed['guide_position'], '8004,8000')
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['quest_catalog'].exists())

    def test_explicit_quest_mode_uses_the_manifest_quest_and_account_guide_setup(self):
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['quest_lifecycle'])
            self.assertTrue(observed['account_login'])
            self.assertIsNotNone(observed['dialogue_catalog'])
            self.assertEqual(observed['quest_catalog'].read_bytes(), (self.source / 'development-quest.json').read_bytes())
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--quest-lifecycle', '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['quest_catalog'].exists())

    def test_conflicting_modes_and_profile_overrides_fail_before_validator_or_game(self):
        for extra in [['--creature-retaliation', 'other.json'], ['--player-respawn', 'other.json'], ['--quest-turn-in-xp', 'other.json'], ['--items', 'other.json'], ['--abilities', 'other.json'],
                      ['--graybox-world'], ['--no-prediction'], ['--transport', 'quinn'],
                      ['--clients', '2'], ['--persistence'], ['--dev-tour'], ['--dev-speed', '8']]:
            with self.subTest(extra=extra), patch.object(bundle, 'snapshot') as snapshot, \
                    patch.object(play, 'play') as launch, redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):
                    play.main(['--gameplay-manifest', str(self.manifest), *extra])
                snapshot.assert_not_called()
                launch.assert_not_called()

    def rewarded_manifest(self):
        name = 'development-quest-turn-in-xp.json'
        shutil.copy2(Path('content/gameplay') / name, self.source / name)
        raw = json.loads(self.manifest.read_text())
        raw.update(schema=2, quest_turn_in_xp=name)
        self.manifest.write_text(json.dumps(raw))
        return self.source / name

    def test_rewarded_bundle_seals_all_eleven_roles_and_cleans_after_source_edits(self):
        source = self.rewarded_manifest()
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.REWARDED_ROLES)
            self.assertEqual(report['manifest_schema'], 2)
            recipe = paths['quest_turn_in_xp'].read_bytes()
            source.write_bytes(b'broken later edit')
            self.assertEqual(paths['quest_turn_in_xp'].read_bytes(), recipe)
        self.assertFalse(any(self.session.iterdir()))

    def test_rewarded_manifest_selects_explicit_mode_and_the_sealed_recipe(self):
        source = self.rewarded_manifest()
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['quest_lifecycle'])
            self.assertTrue(observed['account_login'])
            self.assertEqual(observed['quest_turn_in_xp'].read_bytes(), source.read_bytes())
            self.assertNotEqual(observed['quest_turn_in_xp'].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['quest_turn_in_xp'].exists())

    def test_bad_reward_recipe_is_refused_before_snapshot_or_game(self):
        source = self.rewarded_manifest()
        raw = json.loads(source.read_text())
        raw['experience'] = 0
        source.write_text(json.dumps(raw))
        with self.assertRaises(bundle.BundleError):
            with bundle.snapshot(self.manifest, self.validator, self.session):
                self.fail('Invalid completion recipe reached launch')
        self.assertFalse(self.session.exists())

    def life_manifest(self):
        self.rewarded_manifest()
        raw = json.loads(self.manifest.read_text())
        raw['schema'] = 3
        for role, name in [('creature_retaliation', 'development-creature-retaliation.json'),
                           ('player_respawn', 'development-player-respawn.json')]:
            shutil.copy2(Path('content/gameplay') / name, self.source / name)
            raw[role] = name
        self.manifest.write_text(json.dumps(raw))

    def test_life_bundle_seals_thirteen_roles_and_launcher_passes_the_sealed_pair(self):
        self.life_manifest()
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.LIFE_ROLES)
            self.assertEqual(report['manifest_schema'], 3)
            old = paths['creature_retaliation'].read_bytes()
            (self.source / 'development-creature-retaliation.json').write_bytes(b'later author edit')
            self.assertEqual(paths['creature_retaliation'].read_bytes(), old)
        self.life_manifest()
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            for role in ['creature_retaliation', 'player_respawn']:
                self.assertTrue(observed[role].is_file())
                self.assertNotEqual(observed[role].parent, self.source)
            self.assertTrue(observed['quest_lifecycle'])
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['player_respawn'].exists())
        self.assertFalse(any(self.session.iterdir()))

    def test_life_profile_failure_is_refused_before_launch(self):
        self.life_manifest()
        recipe = self.source / 'development-player-respawn.json'
        raw = json.loads(recipe.read_text())
        raw['stats'] = 'foreign.stats'
        recipe.write_text(json.dumps(raw))
        with patch.object(play, 'play') as launch, self.assertRaises(bundle.BundleError):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        launch.assert_not_called()
        self.assertFalse(self.session.exists())

    def dodge_manifest(self):
        self.life_manifest()
        raw = json.loads(self.manifest.read_text())
        raw.update(schema=4, dodge='development-dodge.json')
        shutil.copy2(Path('content/gameplay/development-dodge.json'), self.source / raw['dodge'])
        self.manifest.write_text(json.dumps(raw))

    def test_dodge_bundle_seals_fourteen_roles_and_passes_one_profile_to_both_children(self):
        self.dodge_manifest()
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.DODGE_ROLES)
            self.assertEqual(report['manifest_schema'], 4)
            old = paths['dodge'].read_bytes()
            (self.source / 'development-dodge.json').write_bytes(b'later edit')
            self.assertEqual(paths['dodge'].read_bytes(), old)
        self.dodge_manifest()
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['dodge_profile'].is_file())
            self.assertNotEqual(observed['dodge_profile'].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['dodge_profile'].exists())
        self.assertFalse(any(self.session.iterdir()))

    def test_invalid_dodge_bundle_never_launches(self):
        self.dodge_manifest()
        raw = json.loads((self.source / 'development-dodge.json').read_text())
        raw['distance_metres'] = 1000
        (self.source / 'development-dodge.json').write_text(json.dumps(raw))
        with patch.object(play, 'play') as launch, self.assertRaises(bundle.BundleError):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        launch.assert_not_called()
        self.assertFalse(self.session.exists())

    def test_melee_bundle_snapshots_weapon_bytes_and_enables_classic(self):
        original=Path('content/gameplay/development-gameplay-classic-melee.json')
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.MELEE_ROLES:
            name=json.loads(original.read_text())[role]
            shutil.copy2(original.parent/name,self.source/name)
        with bundle.snapshot(self.manifest,self.validator,self.session) as (paths,report):
            self.assertEqual(set(paths),bundle.MELEE_ROLES)
            before=paths['melee_weapons'].read_bytes()
            (self.source/'development-melee-weapons.json').write_bytes(b'later author edit')
            self.assertEqual(paths['melee_weapons'].read_bytes(),before)
        shutil.copy2(original.parent/'development-melee-weapons.json',self.source/'development-melee-weapons.json')
        observed={}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['classic'])
            self.assertTrue(observed['melee_weapons'].is_file())
            self.assertIsNone(observed['dodge_profile'])
        with patch.object(play,'play',side_effect=launch),redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest',str(self.manifest),'--gameplay-validator',str(self.validator)])
        self.assertFalse(observed['melee_weapons'].exists())

    def test_physical_bundle_seals_and_forwards_both_profiles(self):
        original = Path('content/gameplay/development-gameplay-classic-physical.json')
        raw = json.loads(original.read_text())
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.PHYSICAL_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.PHYSICAL_ROLES)
            self.assertEqual(report['manifest_schema'], 6)
            before = paths['physical_stats'].read_bytes()
            (self.source / raw['physical_stats']).write_bytes(b'later edit')
            self.assertEqual(paths['physical_stats'].read_bytes(), before)
        shutil.copy2(original.parent / raw['physical_stats'], self.source / raw['physical_stats'])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['classic'])
            for name in ['physical_definitions', 'physical_stats']:
                self.assertTrue(observed[name].is_file())
                self.assertNotEqual(observed[name].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['physical_stats'].exists())

    def test_partial_physical_profiles_refuse_before_children_start(self):
        for profiles in [dict(physical_definitions=Path('definitions.json')),
                         dict(physical_stats=Path('stats.json')),
                         dict(physical_definitions=Path('definitions.json'), physical_stats=Path('stats.json'))]:
            with patch.object(play.subprocess, 'Popen') as child, self.assertRaisesRegex(ValueError, 'Physical damage'):
                play.play(None, Path('.'), 1, Path('.'), **profiles)
            child.assert_not_called()

    def test_ordinary_bundle_seals_and_forwards_its_profile_with_physical_context(self):
        original = Path('content/gameplay/development-gameplay-classic-outcomes.json')
        raw = json.loads(original.read_text())
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.WHITE_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.WHITE_ROLES)
            self.assertEqual(report['manifest_schema'], 7)
            before = paths['white_melee'].read_bytes()
            (self.source / raw['white_melee']).write_bytes(b'later edit')
            self.assertEqual(paths['white_melee'].read_bytes(), before)
        shutil.copy2(original.parent / raw['white_melee'], self.source / raw['white_melee'])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['classic'])
            for name in ['white_melee', 'physical_definitions', 'physical_stats']:
                self.assertTrue(observed[name].is_file())
                self.assertNotEqual(observed[name].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['white_melee'].exists())

    def test_ordinary_profile_without_physical_context_refuses_before_starting_children(self):
        with patch.object(play.subprocess, 'Popen') as child, self.assertRaisesRegex(ValueError, 'Ordinary outcomes require'):
            play.play(None, Path('.'), 1, Path('.'), white_melee=Path('white.json'))
        child.assert_not_called()

    def test_ability_bundle_seals_and_forwards_both_replacement_roles(self):
        original = Path('content/gameplay/development-gameplay-classic-ability-damage.json')
        raw = json.loads(original.read_text())
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.ABILITY_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.ABILITY_ROLES)
            self.assertEqual(report['manifest_schema'], 8)
            for name in ['ability_starting', 'ability_damage']:
                before = paths[name].read_bytes()
                (self.source / raw[name]).write_bytes(b'later edit')
                self.assertEqual(paths[name].read_bytes(), before)
        for name in ['ability_starting', 'ability_damage']:
            shutil.copy2(original.parent / raw[name], self.source / raw[name])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            for name in ['white_melee', 'ability_starting', 'ability_damage']:
                self.assertTrue(observed[name].is_file())
                self.assertNotEqual(observed[name].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['ability_damage'].exists())

    def test_partial_or_unbound_ability_roles_refuse_before_children_start(self):
        for profiles in [dict(ability_starting=Path('starting.json')),
                         dict(ability_damage=Path('damage.json')),
                         dict(ability_starting=Path('starting.json'), ability_damage=Path('damage.json'))]:
            with patch.object(play.subprocess, 'Popen') as child, self.assertRaises(ValueError):
                play.play(None, Path('.'), 1, Path('.'), **profiles)
            child.assert_not_called()

    def test_non_bundle_launcher_retains_existing_direct_profile_selection(self):
        with patch.object(play, 'play') as launch:
            play.main(['--combat'])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertEqual(args['combat_catalog'], Path('content/gameplay/development-abilities.json').resolve())
        self.assertFalse(args['account_login'])

    def test_classic_cli_forwards_movement_preferences_without_enabling_dodge(self):
        controls=self.root/'controls.json'
        shutil.copy2('content/input/classic-controls.json',controls)
        with patch.object(play,'play') as launch,redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest',str(self.manifest),'--gameplay-validator',str(self.validator),'--classic','--controls-file',str(controls)])
        args=PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertTrue(args['classic'])
        self.assertEqual(args['controls_file'],controls)
        self.assertIsNone(args['dodge_profile'])

    def test_classic_launcher_refuses_mixed_or_unqualified_movement_before_starting_children(self):
        base=dict(world=None,players=Path('.'),clients=1,binaries=Path('.'),graybox=True,prediction=True,classic=True)
        for change in [dict(prediction=False),dict(graybox=False),dict(transport='quinn'),dict(dev_tour=True),dict(dev_speed=8),dict(dodge_profile=Path('dodge.json'))]:
            with self.subTest(change=change),patch.object(play.subprocess,'Popen') as child,self.assertRaisesRegex(ValueError,'Classic controls require'):
                play.play(**(base|change))
            child.assert_not_called()
        with patch.object(play.subprocess,'Popen') as child,self.assertRaisesRegex(ValueError,'Controls file requires'):
            play.play(**(base|dict(classic=False,controls_file=Path('controls.json'))))
        child.assert_not_called()

    def test_membership_bundle_snapshots_and_forwards_all_twenty_roles(self):
        original = Path('content/gameplay/development-gameplay-classic-membership.json')
        raw = json.loads(original.read_text())
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.MEMBERSHIP_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.MEMBERSHIP_ROLES)
            self.assertEqual(report['manifest_schema'], 9)
            before = paths['combat_membership'].read_bytes()
            (self.source / raw['combat_membership']).write_bytes(b'later edit')
            self.assertEqual(paths['combat_membership'].read_bytes(), before)
        shutil.copy2(original.parent / raw['combat_membership'], self.source / raw['combat_membership'])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['combat_membership'].is_file())
            self.assertNotEqual(observed['combat_membership'].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['combat_membership'].exists())

    def test_membership_refuses_unbound_launch_before_children_start(self):
        for profiles in [dict(combat_membership=Path('membership.json')),
                         dict(combat_membership=Path('membership.json'), classic=True, ability_starting=Path('starting.json'), ability_damage=Path('damage.json'), white_melee=Path('white.json'))]:
            with patch.object(play.subprocess, 'Popen') as child, self.assertRaises(ValueError):
                play.play(None, Path('.'), 1, Path('.'), **profiles)
            child.assert_not_called()

    def test_credit_bundle_snapshots_and_forwards_all_twenty_one_roles(self):
        original = Path('content/gameplay/development-gameplay-classic-credit.json')
        raw = json.loads(original.read_text())
        self.manifest.write_bytes(original.read_bytes())
        for role in bundle.CREDIT_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.CREDIT_ROLES)
            self.assertEqual(report['manifest_schema'], 10)
            before = paths['creature_credit'].read_bytes()
            (self.source / raw['creature_credit']).write_bytes(b'later edit')
            self.assertEqual(paths['creature_credit'].read_bytes(), before)
        shutil.copy2(original.parent / raw['creature_credit'], self.source / raw['creature_credit'])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['creature_credit'].is_file())
            self.assertNotEqual(observed['creature_credit'].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['creature_credit'].exists())

    def test_credit_refuses_unbound_launch_before_children_start(self):
        for profiles in [dict(creature_credit=Path('credit.json')),
                         dict(creature_credit=Path('credit.json'), classic=True, ability_starting=Path('starting.json'), ability_damage=Path('damage.json'), white_melee=Path('white.json'))]:
            with patch.object(play.subprocess, 'Popen') as child, self.assertRaises(ValueError):
                play.play(None, Path('.'), 1, Path('.'), **profiles)
            child.assert_not_called()

    def test_reset_snapshot_and_launcher_bind_the_exact_new_policy(self):
        original = Path('content/gameplay/development-gameplay-classic-reset.json')
        raw = json.loads(original.read_text())
        shutil.copy2(original, self.manifest)
        for role in bundle.RESET_ROLES:
            shutil.copy2(original.parent / raw[role], self.source / raw[role])
        with bundle.snapshot(self.manifest, self.validator, self.session) as (paths, report):
            self.assertEqual(set(paths), bundle.RESET_ROLES)
            self.assertEqual(report['manifest_schema'], 11)
            before = paths['creature_reset'].read_bytes()
            (self.source / raw['creature_reset']).write_bytes(b'later author edit')
            self.assertEqual(paths['creature_reset'].read_bytes(), before)
        shutil.copy2(original.parent / raw['creature_reset'], self.source / raw['creature_reset'])
        observed = {}
        def launch(*args):
            observed.update(PLAY_SIGNATURE.bind(*args).arguments)
            self.assertTrue(observed['creature_reset'].is_file())
            self.assertNotEqual(observed['creature_reset'].parent, self.source)
        with patch.object(play, 'play', side_effect=launch), redirect_stdout(io.StringIO()):
            play.main(['--gameplay-manifest', str(self.manifest), '--gameplay-validator', str(self.validator)])
        self.assertFalse(observed['creature_reset'].exists())

    def test_reset_without_explicit_credit_refuses_before_child_startup(self):
        with patch.object(play.subprocess, 'Popen') as child, self.assertRaises(ValueError):
            play.play(None, Path('.'), 1, Path('.'), creature_reset=Path('reset.json'))
        child.assert_not_called()


if __name__ == '__main__':
    unittest.main()
