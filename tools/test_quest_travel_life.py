"""Travel key ownership across death, fast respawn and interrupted observations."""
import unittest
from types import SimpleNamespace
from qualify_world_input_os import Desktop as NativeDesktop
from unittest.mock import patch
from qualify_quest_journal_os import LifeGuard, TravelLifeChanged, hold_alive


class Desktop:
    hold_key = NativeDesktop.hold_key

    def __init__(self):
        self.events = []
        self.held_keys = set()
        self.display = 0
        self.x = SimpleNamespace(
            XStringToKeysym=lambda name: name.decode(),
            XKeysymToKeycode=lambda display, name: name,
            XFlush=lambda display: None,
        )
        self.t = SimpleNamespace(XTestFakeKeyEvent=lambda display, key, pressed, delay:
                                 self.events.append((key, bool(pressed))))

    def owned_focus(self):
        pass


class TravelLifeTests(unittest.TestCase):
    def setUp(self):
        self.report = {'combat': {'player_generation': 1, 'player_health': 100}}
        self.desktop = Desktop()

    def test_death_releases_movement_at_the_next_observation(self):
        guard = LifeGuard(lambda: self.report)
        def die(_):
            self.report['combat']['player_health'] = 0
        with patch('qualify_world_input_os.time.sleep', die):
            with self.assertRaises(TravelLifeChanged):
                hold_alive(self.desktop, 'w', .35, guard)
        self.assertEqual(self.desktop.events, [('w', True), ('w', False)])

    def test_fast_respawn_cannot_continue_the_old_turn(self):
        guard = LifeGuard(lambda: self.report)
        def respawn(_):
            self.report['combat']['player_generation'] = 2
        with patch('qualify_world_input_os.time.sleep', respawn):
            with self.assertRaises(TravelLifeChanged):
                hold_alive(self.desktop, 'a', .5, guard)
        self.assertEqual(self.desktop.events, [('a', True), ('a', False)])
        LifeGuard(lambda: self.report).check()

    def test_short_turn_has_no_ui_minimum_hold(self):
        guard = LifeGuard(lambda: self.report)
        with patch('qualify_world_input_os.time.monotonic', side_effect=[0., 0., 0., .02]), \
             patch('qualify_world_input_os.time.sleep') as sleep:
            hold_alive(self.desktop, 'a', .015, guard)
        sleep.assert_called_once_with(.015)
        self.assertEqual(self.desktop.events, [('a', True), ('a', False)])
        self.assertFalse(self.desktop.held_keys)

    def test_dead_start_never_presses_a_key(self):
        guard = LifeGuard(lambda: self.report)
        self.report['combat']['player_health'] = 0
        with self.assertRaises(TravelLifeChanged):
            hold_alive(self.desktop, 'w', .35, guard)
        self.assertEqual(self.desktop.events, [])

    def test_disconnected_observation_still_releases_the_owned_key(self):
        guard = LifeGuard(lambda: self.report)
        with patch('qualify_world_input_os.time.sleep', side_effect=RuntimeError('disconnected')):
            with self.assertRaisesRegex(RuntimeError, 'disconnected'):
                hold_alive(self.desktop, 'd', .5, guard)
        self.assertEqual(self.desktop.events, [('d', True), ('d', False)])


if __name__ == '__main__':
    unittest.main()
