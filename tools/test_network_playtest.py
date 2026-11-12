"""Scenario validation and owned process lifecycle; mocked launches are not netcode evidence."""
import json
import io
import threading
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import network_playtest as play


def document():
    return dict(schema=1, world_identity=list(range(32)), players=[
        dict(client_id=2, feet=[8190., .01, 8192.], network=dict(delay_ms=80, seed=7)),
        dict(client_id=1, feet=[8192., .01, 8192.])])


class ScenarioTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.path = Path(self.folder.name) / 'scenario.json'

    def load(self, value):
        self.path.write_text(json.dumps(value))
        return play.Scenario.load(self.path)

    def test_player_count_and_ids_follow_enabled_pawns(self):
        data = document()
        scenario = self.load(data)
        self.assertEqual([p.client_id for p in scenario.players], [1, 2])
        self.assertEqual(scenario.players[0].network.delay_ms, 0)
        self.assertEqual(scenario.players[1].network.added_rtt_ms, 160)
        data['players'][1]['enabled'] = False
        scenario = self.load(data)
        self.assertEqual([p.client_id for p in scenario.players], [2])
        self.assertEqual(scenario.server_profile()['players'], [dict(client_id=2, feet=(8190., .01, 8192.))])
        self.assertNotIn('network', scenario.server_profile()['players'][0])

    def test_relative_controller_is_resolved_and_not_sent_to_server(self):
        controller = self.path.parent / 'walk.json'
        controller.write_text('{"schema":1,"name":"Walk","steps":[{"duration_ms":100,"forward":1000}]}')
        data = document()
        data['players'][0]['controller'] = 'walk.json'
        scenario = self.load(data)
        self.assertEqual(scenario.players[1].controller, controller.resolve())
        self.assertNotIn('controller', scenario.server_profile()['players'][1])

    def test_invalid_configuration_is_rejected_before_any_child(self):
        bad = []
        for field, value in [('schema', True), ('schema', 2), ('world_identity', [0]*31), ('players', []), ('extra', 1)]:
            data = document(); data[field] = value; bad.append(data)
        for field, value in [('client_id', True), ('client_id', 0), ('client_id', 1), ('feet', [0, float('nan'), 0]),
                             ('feet', [-1, 0, 0]), ('feet', [0, 9999, 0]), ('enabled', 1),
                             ('network', dict(loss_percent=101)), ('network', dict(delay_ms=-1)),
                             ('network', dict(outages=[[10, 100], [20, 10]])), ('network', dict(unknown=1)),
                             ('controller', 'not-implemented')]:
            data = document(); data['players'][0][field] = value; bad.append(data)
        data = document()
        for player in data['players']: player['enabled'] = False
        bad.append(data)
        with patch.object(play.subprocess, 'Popen') as child:
            for value in bad:
                with self.subTest(value=value), self.assertRaises(ValueError):
                    self.load(value)
            child.assert_not_called()
        self.path.write_bytes(b' ' * (play.MAX_SCENARIO_BYTES + 1))
        with self.assertRaises(ValueError): play.Scenario.load(self.path)


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.path = Path(self.folder.name) / 'scenario.json'
        self.path.write_text(json.dumps(document()))
        self.root = Path(self.folder.name)
        self.world, self.art, self.binaries = (self.root / name for name in ('world', 'players', 'bin'))
        for directory in (self.world, self.art, self.binaries): directory.mkdir()
        for path in (self.world / 'authored.world.json', self.art / 'character.glb',
                     self.binaries / 'theia-server', self.binaries / 'theia-client'):
            path.write_bytes(b'Lifecycle fixture only')
        self.output = self.root / 'session'
        self.processes, self.relays = [], []

    def execute(self, *, fail_client=None, stop_startup=False, identity=None, owner_stops_after=None):
        stopped = threading.Event()
        owner = self
        class Child:
            def __init__(self, arguments, **kwargs):
                self.arguments, self.returncode = arguments, None
                is_server = Path(arguments[0]).name == 'theia-server'
                if not is_server and len(owner.processes) == fail_client:
                    raise RuntimeError('Synthetic child launch failure')
                owner.processes.append(self)
                if len(owner.processes) == owner_stops_after:
                    stopped.set()
                if is_server:
                    kwargs['stdout'].write('listening=127.0.0.1:1234 local_tickets=fixture\n')
                    kwargs['stdout'].flush()
                if stop_startup or len(owner.processes) == 3:
                    (owner.output / 'stop').touch()
            def poll(self): return self.returncode
            def terminate(self): self.returncode = 0
            def send_signal(self, _signal): self.returncode = 0
            def wait(self, **_kwargs): return self.returncode
            def kill(self): self.returncode = -9
        class Connection:
            def __init__(self, profile):
                self.profile, self.address, self.error = profile, ('127.0.0.1', 10000 + len(owner.relays)), None
                self.stats, self.closed = {}, False
                owner.relays.append(self)
            def start(self, backend): self.backend = backend
            def close(self): self.closed = True
        checked = SimpleNamespace(returncode=0, stdout=json.dumps(dict(classic_source=identity or list(range(32)))), stderr='')
        with patch.object(play.subprocess, 'Popen', Child), patch.object(play.subprocess, 'run', return_value=checked), patch.object(play, 'Relay', Connection):
            play.run(self.path, self.world, self.art, self.binaries, self.output, stop_requested=stopped.is_set)

    def test_owner_exit_during_client_startup_stops_started_children_and_cancels_rest(self):
        self.execute(owner_stops_after=2)
        self.assertEqual(len(self.processes), 2)
        self.assertTrue(all(p.returncode == 0 for p in self.processes))
        self.assertTrue(all(r.closed for r in self.relays))
        summary = json.loads((self.output / 'session.json').read_text())
        self.assertEqual([p['started'] for p in summary['clients']], [True, False])
        self.assertIn('stop requested', summary['reason'])
        self.assertFalse(list(self.output.glob('tickets-*')))

    def test_owner_exit_before_preflight_creates_no_output_or_processes(self):
        stopped = threading.Event()
        play.watch_owner(io.BytesIO(b''), stopped)
        with patch.object(play.subprocess, 'run') as check, patch.object(play.subprocess, 'Popen') as child:
            play.run(self.path, self.world, self.art, self.binaries, self.output, stop_requested=stopped.is_set)
            check.assert_not_called()
            child.assert_not_called()
        self.assertFalse(self.output.exists())

    def test_owner_exit_during_preflight_never_launches_children(self):
        stopped = threading.Event()
        def checked(*_args, **_kwargs):
            stopped.set()
            return SimpleNamespace(returncode=0, stdout=json.dumps(dict(classic_source=list(range(32)))), stderr='')
        with patch.object(play.subprocess, 'run', side_effect=checked), patch.object(play.subprocess, 'Popen') as child:
            play.run(self.path, self.world, self.art, self.binaries, self.output, stop_requested=stopped.is_set)
            child.assert_not_called()
        self.assertFalse(self.output.exists())

    def test_matching_clients_get_independent_profiles_and_all_children_stop(self):
        self.execute()
        self.assertEqual(len(self.processes), 3)
        self.assertTrue(all(p.returncode == 0 for p in self.processes))
        self.assertTrue(all(r.closed for r in self.relays))
        self.assertEqual([r.profile.delay_ms for r in self.relays], [0, 80])
        for index, process in enumerate(self.processes[1:], 1):
            self.assertTrue(process.arguments[-1].endswith(f'client-{index}.token'))
            self.assertIn(f'127.0.0.1:{10000 + index - 1}', process.arguments)
            self.assertIn('--classic', process.arguments)
        self.assertFalse(list(self.output.glob('tickets-*')))
        self.assertEqual(json.loads((self.output / 'session.json').read_text())['reason'], 'stop requested')

    def test_only_assigned_client_receives_controller_argument(self):
        script = self.root / 'walk.json'
        script.write_text('{}') # Shape validation is delegated to the actual client preflight.
        data = document(); data['players'][0]['controller'] = str(script)
        self.path.write_text(json.dumps(data))
        self.execute()
        self.assertNotIn('--test-controller', self.processes[1].arguments)
        self.assertIn('--test-controller', self.processes[2].arguments)
        self.assertIn(str(script), self.processes[2].arguments)

    def test_partial_launch_failure_cleans_up_first_client_server_and_all_relays(self):
        with self.assertRaisesRegex(RuntimeError, 'Synthetic'):
            self.execute(fail_client=2)
        self.assertEqual(len(self.processes), 2)
        self.assertTrue(all(p.returncode == 0 for p in self.processes))
        self.assertTrue(all(r.closed for r in self.relays))
        self.assertFalse(list(self.output.glob('tickets-*')))
        self.assertEqual(json.loads((self.output / 'session.json').read_text())['reason'], 'failed')

    def test_stop_during_startup_prevents_clients_and_stops_owned_server(self):
        self.execute(stop_startup=True)
        self.assertEqual(len(self.processes), 1)
        self.assertEqual(self.processes[0].returncode, 0)
        self.assertTrue(all(r.closed for r in self.relays))

    def test_foreign_world_starts_nothing_and_does_not_create_output(self):
        with self.assertRaisesRegex(ValueError, 'different authored world'):
            self.execute(identity=[255]*32)
        self.assertFalse(self.processes)
        self.assertFalse(self.output.exists())


if __name__ == '__main__':
    unittest.main()
