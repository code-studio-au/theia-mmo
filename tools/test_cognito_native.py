from pathlib import Path
import shutil
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch


LAUNCHER = Path(__file__).resolve().parents[1] / 'apps/launcher'
sys.path.insert(0, str(LAUNCHER))
import cognito_native
from client_install import InstallError


@unittest.skipUnless(shutil.which('node'), 'requires Node.js')
class NativeSignInTest(unittest.TestCase):
    def sign_in(self, scenario, challenge=None, *, cancelled=lambda: False, seconds=5):
        node = Path(shutil.which('node'))
        root = LAUNCHER / 'native-auth'
        script = root / 'dist/cognito-sign-in.cjs'
        fixture = root / 'test-cognito-response.cjs'
        identity = SimpleNamespace(issuer='https://cognito-idp.ap-southeast-2.amazonaws.com/ap-southeast-2_Example',
                                   client_id='client1')
        start = subprocess.Popen

        def launch(args, **options):
            self.assertEqual(args, [node, script])
            self.assertEqual(options['env'], {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'})
            options['env']['THEIA_COGNITO_TEST'] = scenario
            return start([node, '--require', fixture, script], **options)

        with patch.object(cognito_native, 'NODE', node), patch.object(cognito_native, 'bundle', return_value=script), \
                patch.object(cognito_native.subprocess, 'Popen', side_effect=launch):
            return cognito_native.sign_in(identity, 'tester', 'secret-password', cancelled,
                                          challenge or self.unexpected_challenge, seconds=seconds)

    def unexpected_challenge(self, *_args):
        self.fail('Unexpected challenge')

    def test_srp_yields_only_access_token(self):
        self.assertEqual(self.sign_in('success'),
                         'e30.eyJpYXQiOjE3MDAwMDAwMDAsImV4cCI6MjA1MDAwMDAwMH0.c2ln')

    def test_errors_and_account_setup_link(self):
        with self.assertRaisesRegex(InstallError, '^The username or password was not accepted\\.$'):
            self.sign_in('invalid')
        with self.assertRaisesRegex(InstallError, '^Finish setting up this account using Account help\\.$'):
            self.sign_in('setup')

    def test_temporary_password_and_authenticator_challenge(self):
        prompts = []

        def respond(kind, _message, check):
            check()
            prompts.append(kind)
            return {'new_password': 'new-secret-password', 'totp_code': '123456'}[kind]

        self.assertTrue(self.sign_in('challenges', respond))
        self.assertEqual(prompts, ['new_password', 'totp_code'])

    def test_cancel_and_deadline(self):
        with self.assertRaises(KeyboardInterrupt):
            self.sign_in('success', cancelled=lambda: True)
        with self.assertRaisesRegex(InstallError, 'Sign-in timed out'):
            self.sign_in('wait', seconds=.5)


if __name__ == '__main__':
    unittest.main()
