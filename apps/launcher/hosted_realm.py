"""Cognito browser sign-in and the authenticated native character menu."""
import ipaddress
import os
from pathlib import Path
import re
import ssl
import tempfile
import threading

from account_login import AccountClient, AccountError
from browser_sign_in import SignInConfig, browser_access_token
import client_install
from game_frontend import run_hosted


class HostedRealm:
    def __init__(self, path):
        data = client_install.read_json(path, 16384)
        fields = {'identity', 'address', 'certificate', 'client_version', 'game_arguments'}
        if not isinstance(data, dict) or data.keys() != fields:
            raise client_install.InstallError('Invalid development realm configuration')
        if not isinstance(data['identity'], dict) or data['identity'].keys() != SignInConfig.__dataclass_fields__.keys():
            raise client_install.InstallError('Invalid Cognito sign-in configuration')
        self.identity = SignInConfig(**data['identity'])
        self.identity.validate()
        prefix = r'https://[a-z0-9-]+\.auth\.ap-southeast-2\.amazoncognito\.com'
        domain, _, _ = self.identity.authorization_endpoint.rpartition('/oauth2/authorize')
        if (not re.fullmatch(r'https://cognito-idp\.ap-southeast-2\.amazonaws\.com/ap-southeast-2_[A-Za-z0-9]+', self.identity.issuer)
                or not re.fullmatch(prefix, domain)
                or self.identity.authorization_endpoint != domain + '/oauth2/authorize'
                or self.identity.token_endpoint != domain + '/oauth2/token'
                or self.identity.userinfo_endpoint != domain + '/oauth2/userInfo'):
            raise client_install.InstallError('Development realm requires Sydney Cognito HTTPS endpoints')
        address = data['address']
        try:
            ip, _, port = address.rpartition(':')
            if not ipaddress.IPv4Address(ip).is_global or port != '5000':
                raise ValueError()
        except (AttributeError, ValueError):
            raise client_install.InstallError('Invalid development game endpoint') from None
        self.address = address
        self.origin = 'https://' + ip
        certificate = data['certificate']
        if not isinstance(certificate, str) or Path(certificate).name != certificate:
            raise client_install.InstallError('Use the realm certificate supplied with this configuration')
        self.tls = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        self.tls.minimum_version = ssl.TLSVersion.TLSv1_2
        self.tls.load_verify_locations(cafile=path.parent / certificate)
        self.client_version = client_install.version(data['client_version'])
        self.game_arguments = data['game_arguments']
        if (not isinstance(self.game_arguments, list) or len(self.game_arguments) > 128
                or any(not isinstance(arg, str) or not 0 < len(arg) <= 4096 or '\0' in arg
                       for arg in self.game_arguments)):
            raise client_install.InstallError('Invalid development game configuration')

    def account(self, cancelled=lambda: False):
        return AccountClient(self.origin, cancelled=cancelled, tls=self.tls, game_address=self.address)

    def ready(self):
        return self.account().ready()

    def launch(self, root, window_arguments, cancelled):
        if client_install.active(root)['current'] != self.client_version:
            raise client_install.InstallError(f'Install client {self.client_version} to play this realm')
        account = self.account(cancelled)
        if not account.ready():
            raise client_install.InstallError('The Sydney game server is unavailable')
        directory = None
        try:
            account.use_access_token(browser_access_token(
                self.identity, cancelled, scope='openid theia-development/play', timeout=240,
            ))
            directory = tempfile.TemporaryDirectory(prefix='theia-hosted-game-')
            session = Path(directory.name)
            ticket = session / 'admission.token'
            args = [*self.game_arguments, *window_arguments,
                    '--game-front-end', str(session), '--renet-connect', '127.0.0.1:1', str(ticket)]
            process = client_install.launch(root, args, required_version=self.client_version,
                    env={**os.environ, 'THEIA_HOSTED_GAME_ADDRESS': self.address})
        except (AccountError, KeyboardInterrupt, OSError, client_install.InstallError) as error:
            if directory is not None:
                directory.cleanup()
            account.close()
            raise client_install.InstallError(str(error) or 'Sign-in cancelled') from None

        def maintain_session():
            try:
                if not run_hosted(account, session, ticket, lambda: process.poll() is not None):
                    if process.poll() is None:
                        process.terminate()
            except (AccountError, OSError, ValueError) as error:
                process.theia_hosted_error = str(error)
                if process.poll() is None:
                    process.terminate()
            except KeyboardInterrupt:
                pass
            finally:
                process.wait()
                account.close()
                directory.cleanup()

        worker = threading.Thread(target=maintain_session, name='theia-hosted-account')
        worker.start()
        return process
