"""Lifetime policy for owned identity fixtures; no services or credentials emitted."""
import unittest
from private_identity import Identity


class LifetimePolicy(unittest.TestCase):
    def test_qualification_lifetimes_keep_the_existing_short_bounds(self):
        self.assertEqual(Identity('unused', 'unused').token_seconds, 60)
        self.assertEqual(Identity('unused', 'unused', token_seconds=300).token_seconds, 300)
        for value in [59, 301, 3600, True, '60', float('nan')]:
            with self.assertRaises(ValueError):
                Identity('unused', 'unused', token_seconds=value)

    def test_longer_play_lifetime_requires_explicit_interactive_policy(self):
        for value in [60, 3600, 43200]:
            self.assertEqual(Identity('unused', 'unused', token_seconds=value,
                                      interactive=True).token_seconds, value)
        for value in [59, 43201, True, '3600']:
            with self.assertRaises(ValueError):
                Identity('unused', 'unused', token_seconds=value, interactive=True)
        with self.assertRaises(ValueError):
            Identity('unused', 'unused', token_seconds=3600, interactive=1)


if __name__ == '__main__':
    unittest.main()
