"""Frozen account population launch setup; never starts services or windows."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import account_population_playtest as launcher


class AccountPopulation(unittest.TestCase):
    def fixture(self, root):
        for name in ['package/population.json', 'package/world/authored.world.json',
                     'package/creature.glb', 'players/character.glb']:
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'fixture')
        (root/'package/population.json').write_text('{"schema":1,"creature":"creature.json"}')
        (root/'package/creature.json').write_text('{"schema":3}')
        for _, name in [*launcher.PROFILES, launcher.RESET_PROFILE, launcher.THREAT_PROFILE]:
            path = root/'content/gameplay'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
        return argparse.Namespace(package=root/'package', players=root/'players', binaries=root/'bin',
                                  output=root/'session', window_size='1280x720', character_id=None)

    def test_profiles_are_frozen_and_exact_paths_drive_the_owned_account_launcher(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            command = launcher.prepare(args, root)
            report = json.loads((args.output/'launch.json').read_text())
            self.assertEqual(len(report['profiles']), 16)
            self.assertIn('--character-picker', command)
            self.assertIn('--stop-on-stdin-eof', command)
            self.assertIn('--account-login', command)
            self.assertNotIn('--creature-profile', command)
            self.assertNotIn('--test-controller', command)
            self.assertNotIn('--public-creature-attacks', command)
            self.assertFalse(report['public_creature_attacks'])
            self.assertEqual(command[command.index('--session-directory')+1], str(args.output/'runtime'))
            for record in report['profiles']:
                path = args.output/record['file']
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), record['sha256'])
                self.assertEqual(command.count(record['option']), 1)
                self.assertEqual(command[command.index(record['option'])+1], str(path))
                (root/'content/gameplay'/path.name).write_bytes(b'changed source')
                self.assertNotEqual(path.read_bytes(), b'changed source')

    def test_selected_catalogues_are_frozen_and_enable_shared_corpse_pickup(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.items = root/'items.json'
            args.reward_catalog = root/'rewards.json'
            args.items.write_text('{"schema":2}')
            args.reward_catalog.write_text('{"rewards":[{"scope":"per_creature_life"}]}')
            command = launcher.prepare(args, root)
            self.assertIn('--creature-loot', command)
            self.assertEqual(command.count('--items'), 1)
            self.assertEqual(command.count('--reward-catalog'), 1)
            self.assertEqual((args.output/'profiles/selected-items.json').read_bytes(), args.items.read_bytes())
            self.assertEqual((args.output/'profiles/selected-rewards.json').read_bytes(), args.reward_catalog.read_bytes())
            self.assertTrue(json.loads((args.output/'launch.json').read_text())['creature_loot'])

    def test_pursuing_definition_freezes_and_selects_the_shared_return_reset(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            (args.package/'creature.json').write_text('{"schema":3,"aggro":{"perception_metres":12}}')
            command = launcher.prepare(args, root)
            report = json.loads((args.output/'launch.json').read_text())
            self.assertTrue(report['creature_reset'])
            self.assertEqual(len(report['profiles']), 18)
            self.assertTrue(report['creature_threat'])
            self.assertTrue(report['public_creature_attacks'])
            self.assertEqual(command.count('--public-creature-attacks'), 1)
            self.assertEqual(command.count('--creature-threat'), 1)
            self.assertEqual(command.count('--creature-reset'), 1)
            path = Path(command[command.index('--creature-reset')+1])
            self.assertEqual(path.read_bytes(), (root/'content/gameplay'/launcher.RESET_PROFILE[1]).read_bytes())

    def test_explicit_journal_freezes_contact_and_selects_conditional_loot_only_when_authored(self):
        for conditional in [False, True]:
            with self.subTest(conditional=conditional), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                args = self.fixture(root)
                args.quest_catalog, args.dialogue_catalog = root/'quests.json', root/'dialogue.json'
                args.guide_position = '8193,0.008,8191'
                args.quest_areas = root/'areas.json'
                args.quest_areas.write_text('{"schema":1,"areas":[]}')
                args.quest_catalog.write_text('{"schema":3,"quests":[]}')
                args.dialogue_catalog.write_text('{"schema":1}')
                args.items, args.reward_catalog = root/'items.json', root/'rewards.json'
                args.items.write_text('{"schema":2}')
                args.reward_catalog.write_text(json.dumps({'rewards': [dict(scope='per_creature_life', quest_required=conditional)]}))
                command = launcher.prepare(args, root)
                report = json.loads((args.output/'launch.json').read_text())
                self.assertEqual(len(report['profiles']), 19)
                self.assertTrue(report['quest_areas'])
                self.assertEqual(Path(command[command.index('--quest-areas')+1]).read_bytes(), args.quest_areas.read_bytes())
                self.assertTrue(report['quest_journal'] and report['quest_items'])
                self.assertEqual(report['quest_loot'], conditional)
                self.assertEqual('--quest-loot' in command, conditional)
                self.assertIn('--quest-journal', command)
                self.assertIn('--quest-items', command)
                self.assertEqual(command[command.index('--guide-position')+1], args.guide_position)
                for flag, source in [('--quest-catalog', args.quest_catalog), ('--dialogue-catalog', args.dialogue_catalog)]:
                    snapshot = Path(command[command.index(flag)+1])
                    original = source.read_bytes()
                    source.write_bytes(b'changed source')
                    self.assertEqual(snapshot.read_bytes(), original)

    def test_partial_quest_contact_refuses_before_creating_output(self):
        for missing in ['quest_catalog', 'dialogue_catalog', 'guide_position']:
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                args = self.fixture(root)
                args.quest_catalog = root/'missing-quests.json'
                args.dialogue_catalog = root/'missing-dialogue.json'
                args.guide_position = '0,0'
                setattr(args, missing, None)
                with self.assertRaisesRegex(ValueError, 'together'):
                    launcher.prepare(args, root)
                self.assertFalse(args.output.exists())

    def test_conditional_rewards_without_journal_refuse_before_creating_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.items, args.reward_catalog = root/'items.json', root/'rewards.json'
            args.items.write_text('{"schema":2}')
            args.reward_catalog.write_text('{"rewards":[{"scope":"per_creature_life","quest_required":true}]}')
            with self.assertRaisesRegex(ValueError, 'inventory-bound quest journal'):
                launcher.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_quest_areas_without_journal_refuse_before_creating_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.quest_areas = root/'missing-areas.json'
            with self.assertRaisesRegex(ValueError, 'Quest areas require'):
                launcher.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_oversized_quest_contact_is_bounded_before_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.quest_catalog, args.dialogue_catalog = root/'quests.json', root/'dialogue.json'
            args.guide_position = '0,0'
            args.quest_catalog.write_bytes(b'x'*(1024*1024+1))
            args.dialogue_catalog.write_text('{}')
            with self.assertRaisesRegex(ValueError, 'byte budget'):
                launcher.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_behavior_inputs_reject_escape_and_oversized_data_before_output(self):
        for value in ['../outside.json', '/tmp/outside.json', 'a\\outside.json', 'x'*257]:
            with self.subTest(path=value), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                args = self.fixture(root)
                (args.package/'population.json').write_text(json.dumps({'creature':value}))
                with self.assertRaises(ValueError):
                    launcher.prepare(args, root)
                self.assertFalse(args.output.exists())
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            (args.package/'creature.json').write_bytes(b' '*4097)
            with self.assertRaisesRegex(ValueError, 'byte budget'):
                launcher.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_partial_catalogue_selection_refuses_without_creating_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.items = root/'missing.json'
            with self.assertRaisesRegex(ValueError, 'together'):
                launcher.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_existing_output_is_not_replaced_and_explicit_character_excludes_picker(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.character_id = '74686569-6100-0000-0000-000000000101'
            command = launcher.prepare(args, root)
            self.assertNotIn('--character-picker', command)
            self.assertEqual(command[command.index('--character-id')+1], args.character_id)
            snapshot = (args.output/'launch.json').read_bytes()
            with self.assertRaises(FileExistsError):
                launcher.prepare(args, root)
            self.assertEqual((args.output/'launch.json').read_bytes(), snapshot)

    def test_missing_or_oversized_dependencies_refuse_before_creating_output(self):
        for oversized in (False, True):
            with self.subTest(oversized=oversized), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                args = self.fixture(root)
                if oversized:
                    (root/'content/gameplay'/launcher.PROFILES[0][1]).write_bytes(b'x'*(1024*1024+1))
                else:
                    (args.package/'population.json').unlink()
                with self.assertRaises(ValueError):
                    launcher.prepare(args, root)
                self.assertFalse(args.output.exists())


if __name__ == '__main__':
    unittest.main()
