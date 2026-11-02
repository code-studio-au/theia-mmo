"""Service configuration isolation at the native child handoff."""
import unittest
from launch_environment import client_environment


class LaunchEnvironment(unittest.TestCase):
    def test_database_and_provider_credentials_are_removed_without_changing_parent(self):
        source = {key: 'owned-private-value' for key in [
            'DATABASE_URL', 'THEIA_DATABASE_URL', 'PGPASSWORD', 'PGPASSFILE',
            'PGSERVICE', 'PGSERVICEFILE', 'PGSSLKEY', 'PGUSER', 'PGHOST',
            'THEIA_OIDC_ISSUER', 'THEIA_OIDC_CLIENT_ID', 'THEIA_OIDC_CLIENT_SECRET',
            'THEIA_ACCOUNT_BIND', 'THEIA_TEST_DATABASE_URL', 'THEIA_TEST_BROWSER_PASSWORD',
        ]}
        before = source.copy()
        self.assertEqual(client_environment(source), {})
        self.assertEqual(source, before)

    def test_os_render_and_native_qualification_settings_remain_available(self):
        source = {'PATH': '/owned/bin', 'LD_LIBRARY_PATH': '/owned/lib',
                  'DISPLAY': ':0', 'XDG_RUNTIME_DIR': '/owned/runtime',
                  'THEIA_WINDOW_SIZE': '3440x1440', 'THEIA_GRAYBOX_QUALIFY': '/owned/proof',
                  'THEIA_ZONE_QUALIFY': '1', 'THEIA_PERFORMANCE_UNCAPPED': '1'}
        self.assertEqual(client_environment(source), source)
        self.assertIsNot(client_environment(source), source)

    def test_windows_case_variants_do_not_bypass_service_filter(self):
        self.assertEqual(client_environment({'pgpassfile': 'private',
                                            'Theia_Oidc_Client_Secret': 'private',
                                            'database_url': 'private',
                                            'Path': 'preserved'}), {'Path': 'preserved'})


if __name__ == '__main__':
    unittest.main()
