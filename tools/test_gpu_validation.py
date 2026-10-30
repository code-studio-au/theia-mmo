"""GPU evidence may classify the documented resize race, never arbitrary errors."""
import unittest
from gpu_validation import SURFACE_RESIZE, check_gpu_diagnostics

KNOWN = f'{SURFACE_RESIZE}: Validation Error: vkCreateSwapchainKHR(): imageExtent (800, 600)'


class GpuValidation(unittest.TestCase):
    def test_ordinary_checks_reject_even_the_known_diagnostic(self):
        self.assertEqual(check_gpu_diagnostics('renderer ready'), 0)
        with self.assertRaises(AssertionError):
            check_gpu_diagnostics(KNOWN)

    def test_fullscreen_counts_one_specific_surface_diagnostic(self):
        self.assertEqual(check_gpu_diagnostics(KNOWN, allow_surface_resize=True), 1)

    def test_other_errors_and_mixed_vuids_still_fail(self):
        for line in ['Validation Error without an ID', 'panicked at renderer', 'GPU device lost',
                     KNOWN + ' VUID-other-00001', KNOWN.replace('07781', '07782'),
                     SURFACE_RESIZE + ' unrelated message']:
            with self.subTest(line=line), self.assertRaises(AssertionError):
                check_gpu_diagnostics(line, allow_surface_resize=True)

    def test_persistent_diagnostics_do_not_pass_as_a_startup_race(self):
        with self.assertRaises(AssertionError):
            check_gpu_diagnostics('\n'.join([KNOWN] * 3), allow_surface_resize=True)


if __name__ == '__main__':
    unittest.main()
