import copy
import unittest
from prepare_standard_lighting_preview import supported


class StandardLightingTests(unittest.TestCase):
    def test_only_selective_lights_are_omitted_without_rewriting_source(self):
        source = {'lights': [
            {'id': 'point', 'mask': 4294967295, 'intensity': 3},
            {'id': 'ground', 'mask': 4, 'intensity': 8},
            {'id': 'disabled-mask', 'mask': 0, 'intensity': 9}],
            'ambient': {'brightness': 123}, 'cookies': ['original']}
        before = copy.deepcopy(source)
        actual, omitted = supported(source)
        self.assertEqual(source, before)
        self.assertEqual(actual['lights'], [source['lights'][0]])
        self.assertEqual(omitted, source['lights'][1:])
        self.assertEqual(actual['ambient'], source['ambient'])
        self.assertEqual(actual['cookies'], source['cookies'])
        actual['lights'][0]['intensity'] = 100
        self.assertEqual(source, before)

    def test_an_empty_supported_subset_is_rejected(self):
        for lights in [[], [{'mask': 4}], [{'mask': 0}]]:
            with self.assertRaises(ValueError):
                supported({'lights': lights})
