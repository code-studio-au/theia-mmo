"""Linux CLI preflight against built binaries, with display access removed.

Only invalid/incomplete configurations are invoked. These checks neither start
normal gameplay nor qualify native UI, Windows execution or database admission.
"""
import os
from pathlib import Path
import subprocess
import sys
import unittest


@unittest.skipUnless(sys.platform == 'linux', 'Display-isolated CLI checks are Linux-only')
class QuestItemRewardCli(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.client = Path(os.environ.get('THEIA_TEST_CLIENT_BINARY', 'target/debug/theia-client')).resolve()
        cls.server = Path(os.environ.get('THEIA_TEST_SERVER_BINARY', 'target/debug/theia-server')).resolve()
        if not cls.client.is_file() or not cls.server.is_file():
            raise unittest.SkipTest('Build current debug client and server binaries first')
        cls.environment = {k: v for k, v in os.environ.items()
                           if k.upper() not in ('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR')
                           and not k.upper().startswith(('THEIA_', 'PG'))}
        cls.reward = 'content/quests/development-quest-item-rewards.json'
        cls.documents = ['--document-catalog', 'content/quests/development-documents.json']
        cls.quest = ['--quest-journal', '--quest-catalog', 'content/quests/development-journal-documents.json',
                     '--item-catalog', 'content/gameplay/development-document-items.json']

    def refusal(self, executable, arguments, expected):
        # Empty certificate path cannot be written if a regression starts a
        # transport; subprocess.run also kills/reaps the child on timeout.
        prefix = ['127.0.0.1:0', ''] if executable == self.server else []
        result = subprocess.run([str(executable), *prefix, *arguments], env=self.environment,
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(expected, result.stderr)
        self.assertNotIn('listening=', result.stdout)

    def test_client_missing_or_unbound_reward_policy_refuses_before_window_creation(self):
        for arguments in [['--quest-item-rewards'], ['--quest-item-rewards', self.reward]]:
            with self.subTest(arguments=arguments):
                self.refusal(self.client, arguments, 'Fixed quest rewards require journal, quest and item catalogues')

    def test_client_configuration_report_is_exclusive_and_has_no_fallback_window(self):
        for arguments, expected in [
            (['--configuration-report'], 'Configuration report requires exclusive'),
            (['--configuration-report', '--configuration-report'], 'Choose configuration report once'),
            (['--configuration-report', '--unknown'], 'Configuration report requires exclusive'),
            (['--configuration-report', '--renet-connect', '127.0.0.1:1', 'unused.token'], 'Configuration report requires exclusive'),
        ]:
            with self.subTest(arguments=arguments):
                self.refusal(self.client, arguments, expected)

    def test_server_missing_unbound_or_duplicate_reward_policy_refuses_before_endpoint(self):
        for arguments, expected in [
            (['--quest-item-rewards'], 'Missing fixed quest reward catalogue'),
            (['--quest-item-rewards', self.reward], 'Fixed quest rewards require journal, quest and item catalogues'),
            (['--quest-item-rewards', self.reward, '--quest-item-rewards', self.reward], 'Choose one fixed quest reward catalogue'),
        ]:
            with self.subTest(arguments=arguments):
                self.refusal(self.server, arguments, expected)

    def test_server_reward_prerequisites_are_checked_after_all_flag_orders(self):
        rewards = ['--quest-item-rewards', self.reward]
        for arguments in [rewards + self.documents + self.quest,
                          self.documents + self.quest + rewards,
                          self.quest + rewards + self.documents]:
            with self.subTest(arguments=arguments):
                self.refusal(self.server, arguments,
                             'Journal requires catalog, dialogue and predicted account login')


if __name__ == '__main__':
    unittest.main()
