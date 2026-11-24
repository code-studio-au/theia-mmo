import sys
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps' / 'launcher'))

from realm_gateway import GatewayUnavailable, RealmGateway
from client_install import InstallError


class RealmPlay(unittest.TestCase):
    def setUp(self):
        self.gateway = RealmGateway('https://abc123def4.execute-api.ap-southeast-2.amazonaws.com')
        self.progress = Mock()
        self.ready = Mock(return_value=True)

    def wait(self, cancelled=lambda: False):
        return self.gateway.wait_ready('token', 'join', {'client': 'test'}, cancelled,
                                       self.progress, self.ready, seconds=1, interval=0)

    def test_waits_through_shutdown_and_retries_with_one_reservation(self):
        self.gateway.request = Mock(side_effect=[
            {'state': 'shutting_down'}, GatewayUnavailable(), {'state': 'starting'},
            {'state': 'initialising'}, {'state': 'ready'}, {'state': 'ready'},
        ])
        self.ready.side_effect = [False, True]
        self.wait()
        self.assertEqual(self.ready.call_count, 2)
        for request in self.gateway.request.call_args_list:
            self.assertEqual(request.args, ('POST', 'wake', 'token',
                                            {'reservation': 'join', 'build': {'client': 'test'}}))

    def test_maintenance_and_cancellation_never_open_game(self):
        self.gateway.request = Mock(return_value={'state': 'maintenance'})
        with self.assertRaisesRegex(InstallError, 'maintenance'):
            self.wait()
        self.gateway.request.reset_mock()
        with self.assertRaisesRegex(InstallError, 'cancelled'):
            self.wait(lambda: True)
        self.gateway.request.assert_not_called()
        self.ready.assert_not_called()

    def test_timeout_does_not_check_account_service_when_still_starting(self):
        self.gateway.request = Mock(return_value={'state': 'starting'})
        with patch('realm_gateway.time.monotonic', side_effect=[0, 0, 1, 1, 1]):
            with self.assertRaisesRegex(InstallError, 'time'):
                self.wait()
        self.ready.assert_not_called()

    def test_only_sydney_execute_api_origin_can_receive_token(self):
        for url in ('https://example.com', self.gateway.origin + '/path',
                    self.gateway.origin + '@example.com', self.gateway.origin.replace('https:', 'http:')):
            with self.subTest(url=url), self.assertRaises(InstallError):
                RealmGateway(url)


if __name__ == '__main__':
    unittest.main()
