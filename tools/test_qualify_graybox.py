"""Map-checker regressions; synthetic report data, not multiplayer evidence."""
import copy
import unittest

from qualify_graybox import check_minimap_traversal


def fixture():
    reports = []
    for index in range(100):
        position = [100. + index * 160., 0., 8000.]
        reports.append({
            'seconds': 8. + index,
            'position': [position[0] - 25.6, 0., position[2]],
            'rendered_position': position,
            'rendered_forward': [.8, -.6],
            'rebases': index,
            'minimap': {
                'visible': True, 'centre': position,
                'player_point': [88., 88.], 'arrow_forward': [.8, -.6],
                'texture_bytes': 17408, 'resident_tiles_read': 81,
                'known_pixels': 4096, 'bounds': [1716., 12., 192., 232.],
            },
        })
    result = {
        'render_frame': {'revision': 125},
        'minimap': {
            'visible': True, 'known_pixels': 2048, 'raster_updates': 400,
            'max_raster_us': 400, 'max_present_us': 450,
        },
    }
    return result, reports


class MinimapCheckerTests(unittest.TestCase):
    def test_accepts_exact_display_alignment_despite_authority_prediction_lead(self):
        result, reports = fixture()
        proof = check_minimap_traversal(result, reports, [1920, 1200])
        self.assertTrue(proof['passed'])
        self.assertEqual(proof['max_map_position_error_metres'], 0)

    def test_refuses_map_misalignment_and_arrow_mismatch(self):
        result, reports = fixture()
        shifted = copy.deepcopy(reports)
        shifted[30]['minimap']['centre'][0] += .01
        # Keep the independently observed player fixed.
        shifted[30]['rendered_position'] = reports[30]['rendered_position']
        with self.assertRaises(AssertionError):
            check_minimap_traversal(result, shifted, [1920, 1200])
        rotated = copy.deepcopy(reports)
        rotated[30]['minimap']['arrow_forward'] = [1., 0.]
        with self.assertRaises(AssertionError):
            check_minimap_traversal(result, rotated, [1920, 1200])


if __name__ == '__main__':
    unittest.main()
