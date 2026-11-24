"""Input helper safety checks with no X connection or injected input."""
import unittest
from unittest.mock import Mock, patch
from qualify_world_input_os import Desktop


class Holds(unittest.TestCase):
    def desktop(self):
        desktop = Desktop.__new__(Desktop)
        desktop.display = object()
        desktop.x = Mock()
        desktop.t = Mock()
        desktop.x.XKeysymToKeycode.return_value = 38
        desktop.held_keys = set()
        desktop.owned_focus = Mock()
        return desktop

    def test_exception_releases_pressed_key(self):
        desktop = self.desktop()
        with patch('qualify_world_input_os.time.sleep', side_effect=RuntimeError('interrupted')):
            with self.assertRaises(RuntimeError):
                desktop.hold_key('a', .05)
        self.assertEqual(desktop.held_keys, set())
        self.assertEqual([call.args[2] for call in desktop.t.XTestFakeKeyEvent.call_args_list], [1, 0])

    def test_invalid_duration_and_foreign_focus_emit_no_input(self):
        for duration in [0, -1, 1.01, float('nan'), float('inf')]:
            desktop = self.desktop()
            with self.assertRaises(AssertionError):
                desktop.hold_key('a', duration)
            desktop.t.XTestFakeKeyEvent.assert_not_called()
        desktop = self.desktop()
        desktop.owned_focus.side_effect = AssertionError('foreign focus')
        with self.assertRaises(AssertionError):
            desktop.hold_key('a', .05)
        desktop.t.XTestFakeKeyEvent.assert_not_called()
        self.assertEqual(desktop.held_keys, set())


if __name__ == '__main__':
    unittest.main()
