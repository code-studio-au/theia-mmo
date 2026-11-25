"""Authenticated Sydney lifecycle API. Used only by the hosted Play worker."""
import json
import re
import time
import urllib.error
import urllib.request

from account_login import _NoRedirect
from client_install import InstallError


class GatewayUnavailable(InstallError):
    pass


class RealmGateway:
    def __init__(self, origin):
        if not isinstance(origin, str) or not re.fullmatch(
                r'https://[a-z0-9]{10}\.execute-api\.ap-southeast-2\.amazonaws\.com', origin):
            raise InstallError('Invalid Sydney gateway endpoint')
        self.origin = origin
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())

    def request(self, method, route, token, body=None):
        request = urllib.request.Request(self.origin + '/v1/realm/' + route,
                None if body is None else json.dumps(body).encode(),
                {'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'},
                method=method)
        try:
            with self.opener.open(request, timeout=7) as response:
                raw = response.read(4097)
            if len(raw) > 4096:
                raise ValueError()
            result = json.loads(raw)
            if not isinstance(result, dict):
                raise ValueError()
            return result
        except urllib.error.HTTPError as error:
            with error:
                code = error.code
            if code in (401, 403):
                raise InstallError('Sign in with an approved development tester account') from None
            if code == 409:
                if route == 'reservation':
                    return None
                raise InstallError('Install a compatible game release before playing') from None
            if code == 429 or code >= 500:
                raise GatewayUnavailable('The realm gateway is temporarily unavailable') from None
            raise InstallError('The realm gateway refused the request') from None
        except (OSError, ValueError):
            raise GatewayUnavailable('Could not reach the realm gateway') from None

    def wait_ready(self, token, reservation, build, cancelled, progress, account_ready,
                   *, seconds=300, interval=5):
        deadline = time.monotonic() + seconds
        labels = {
            'offline': 'Waiting to start the Sydney realm…',
            'starting': 'Starting the Sydney host…',
            'initialising': 'Waiting for Millhaven and its database…',
            'ready': 'Checking the game connection…',
            'shutting_down': 'Waiting for the previous session to shut down…',
        }
        while not cancelled() and time.monotonic() < deadline:
            try:
                state = self.request('POST', 'wake', token, {
                    'reservation': reservation, 'build': build,
                }).get('state')
            except GatewayUnavailable:
                progress('Retrying the Sydney realm gateway…')
            else:
                if state == 'maintenance':
                    raise InstallError('The Sydney realm is in maintenance')
                if state == 'error' or state not in labels:
                    raise InstallError('The Sydney realm could not start')
                progress(labels[state])
                if state == 'ready' and account_ready():
                    return
            until = min(deadline, time.monotonic() + interval)
            while not cancelled() and time.monotonic() < until:
                time.sleep(min(.25, max(0, until - time.monotonic())))
        if cancelled():
            raise InstallError('Play cancelled')
        raise InstallError('The Sydney realm did not become ready in time')

    def release(self, token, reservation):
        try:
            self.request('DELETE', 'reservation', token, {'reservation': reservation})
        except InstallError:
            # Server-side expiry covers interrupted or expired credentials.
            pass
