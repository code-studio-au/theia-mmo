"""Desktop panel bounds, independent of a running window manager."""
import unittest
from unittest.mock import patch
import playtest_windows as windows


class WorkArea(unittest.TestCase):
    def test_active_desktop_and_negative_monitor_origin(self):
        self.assertEqual(windows.parse_work_area(
            '0 - DG: 3440x1440 VP: 0,0 WA: 0,0 3440x1440 inactive\n'
            '1 * DG: 6880x1440 VP: 0,0 WA: -3440,40 6880x1400 current'),
            (-3440, 40, 6880, 1400))

    def test_missing_or_invalid_area_is_not_forwarded(self):
        for value in ['', '0 * WA: N/A', '0 * WA: 0,0 3440x0', '0 * WA: 0,0 999999x1440']:
            self.assertIsNone(windows.parse_work_area(value))
        with patch.object(windows, 'desktop_work_area', return_value=None):
            self.assertEqual(windows.work_area_options(), [])

    def test_physical_area_reaches_the_client(self):
        with patch.object(windows, 'desktop_work_area', return_value=(0, 0, 3440, 1400)):
            self.assertEqual(windows.work_area_options(), ['--test-work-area', '0,0,3440,1400'])

    def test_solo_and_tiled_launches_respect_reserved_desktop_panels(self):
        with patch.object(windows, 'desktop_work_area', return_value=(0, 0, 3440, 1400)):
            for clients in (1, 2, 3, 4):
                for index in range(clients):
                    self.assertEqual(windows.client_window_options(index, clients),
                                     ['--test-window', f'{index + 1}/{clients}',
                                      '--test-work-area', '0,0,3440,1400'])
            self.assertEqual(windows.client_window_options(0, 1, test_window='4/4'),
                             ['--test-window', '4/4', '--test-work-area', '0,0,3440,1400'])

    def test_fullscreen_and_large_groups_keep_their_existing_window_mode(self):
        with patch.object(windows, 'desktop_work_area') as query:
            self.assertEqual(windows.client_window_options(0, 1, fullscreen=True), [])
            self.assertEqual(windows.client_window_options(0, 5), [])
            query.assert_not_called()

    def test_missing_work_area_still_uses_bevy_monitor_layout(self):
        with patch.object(windows, 'desktop_work_area', return_value=None):
            self.assertEqual(windows.client_window_options(0, 1), ['--test-window', '1/1'])
