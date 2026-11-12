"""Opt-in actual headless server CLI checks against a locally exported world."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

from launch_environment import client_environment


@unittest.skipUnless(os.environ.get('THEIA_AUTHORED_WORLD') and os.environ.get('THEIA_TEST_PLAYERS_PROFILE'),
                     'requires a local authored world and matching test-player profile')
class ServerCLI(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='theia-test-player-cli-')
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name)
        self.profile = self.folder / 'players.json'
        self.profile.write_bytes(Path(os.environ['THEIA_TEST_PLAYERS_PROFILE']).read_bytes())
        self.binary = Path(os.environ.get('THEIA_SERVER_BINARY', 'target/debug/theia-server')).resolve()
        self.environment = client_environment(os.environ)

    def command(self, extra=()):
        return [str(self.binary), '127.0.0.1:0', str(self.folder / 'unused.der'),
                '--authored-world', str(Path(os.environ['THEIA_AUTHORED_WORLD']).resolve()),
                '--prediction', '--classic', '--renet-tickets', str(self.folder / 'tickets'),
                '--test-players', str(self.profile), *extra]

    def refusal(self, command):
        result = subprocess.run(command, env=self.environment, capture_output=True, text=True, timeout=20)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('listening=', result.stdout)
        self.assertFalse((self.folder / 'tickets').exists(), 'Refused configuration must not issue tickets')
        return result.stderr

    def test_profile_checks_complete_before_ticket_issue(self):
        original = json.loads(self.profile.read_bytes())
        for kind in ('wrong-world', 'unsupported', 'overlap', 'duplicate', 'too-large'):
            with self.subTest(kind=kind):
                document = json.loads(json.dumps(original))
                if kind == 'wrong-world':
                    document['world_identity'][0] ^= 1
                elif kind == 'unsupported':
                    document['players'][0]['feet'][1] = 1000
                elif kind == 'overlap':
                    document['players'][0]['feet'][1] = -1
                elif kind == 'duplicate':
                    document['players'] = [document['players'][0]] * 2
                self.profile.write_text(' ' * (16 * 1024 + 1) if kind == 'too-large' else json.dumps(document))
                self.refusal(self.command())

    def test_test_mode_refuses_account_persistence_and_other_control_modes(self):
        for extra in (['--account-login'], ['--dev-persistence'], ['--dev-speed', '8'], ['--dev-tour']):
            with self.subTest(extra=extra):
                self.assertIn('Test players require local-ticket Classic', self.refusal(self.command(extra)))
        command = self.command()
        command.remove('--classic')
        self.assertIn('Test players require local-ticket Classic', self.refusal(command))
        command = self.command()
        command[1] = '0.0.0.0:0'
        self.assertIn('Test players require local-ticket Classic', self.refusal(command))

    def test_valid_profile_starts_and_shuts_down_without_mutating_input(self):
        before = self.profile.read_bytes()
        log = self.folder / 'server.log'
        with log.open('w') as output:
            process = subprocess.Popen(self.command(), env=self.environment, stdout=output, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 20
                while 'listening=' not in log.read_text():
                    self.assertIsNone(process.poll(), log.read_text())
                    self.assertLess(time.monotonic(), deadline, log.read_text())
                    time.sleep(.02)
                self.assertTrue((self.folder / 'tickets/client-1.token').is_file())
                self.assertTrue((self.folder / 'tickets/client-2.token').is_file())
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
            self.assertEqual(process.returncode, 0, log.read_text())
        self.assertEqual(self.profile.read_bytes(), before)
        stats = next(json.loads(line.removeprefix('transport_summary ')) for line in log.read_text().splitlines()
                     if line.startswith('transport_summary '))
        self.assertEqual(stats['test_player_admissions'], 0)
        self.assertEqual(stats['persistent_spawn_admissions'], 0)


if __name__ == '__main__':
    unittest.main()
