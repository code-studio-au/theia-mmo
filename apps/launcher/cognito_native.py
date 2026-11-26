"""Run Cognito SRP with the packaged SDK over private, bounded process pipes."""
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import time

from client_install import InstallError

NODE = Path('/usr/bin/node')
ROOT = Path(__file__).resolve().parent
MAX_REPLY = 16384
PROMPTS = {
    'new_password': 'Choose a new password for this tester account.',
    'sms_code': 'Enter the code sent to your phone.',
    'totp_code': 'Enter the code from your authenticator app.',
}
ERRORS = {
    'NotAuthorizedException': 'The username or password was not accepted.',
    'UserNotFoundException': 'The username or password was not accepted.',
    'PasswordResetRequiredException': 'This account needs a password reset. Open Account help.',
    'UserNotConfirmedException': 'The account is not ready. Contact the tester administrator.',
    'CodeMismatchException': 'The verification code was not accepted. Sign in again to retry.',
    'ExpiredCodeException': 'The verification code expired. Sign in again to retry.',
    'InvalidPasswordException': 'The new password did not meet the account requirements. Try again.',
    'TooManyRequestsException': 'Too many sign-in attempts. Wait and try again.',
    'LimitExceededException': 'Too many sign-in attempts. Wait and try again.',
    'BrowserRequired': 'Finish setting up this account using Account help.',
}
UNAVAILABLE = 'Native sign-in is unavailable. Try again or update the launcher.'


def bundle():
    packaged = ROOT / 'cognito-sign-in.cjs'
    return packaged if packaged.is_file() else ROOT / 'native-auth/dist/cognito-sign-in.cjs'


def available():
    return NODE.is_file() and bundle().is_file()


def sign_in(identity, username, password, cancelled, request_challenge, *, seconds=180):
    if not available():
        raise InstallError(UNAVAILABLE)
    if not username.strip() or not password or len(username) > 128 or len(password) > 256:
        raise InstallError('Enter your username and password.')
    deadline = time.monotonic() + seconds
    pending = bytearray()

    def check():
        if cancelled():
            raise KeyboardInterrupt
        if time.monotonic() >= deadline:
            raise InstallError('Sign-in timed out. Try again.')

    def read(process, watch):
        while b'\n' not in pending:
            check()
            if not watch.select(.1):
                continue
            block = os.read(process.stdout.fileno(), MAX_REPLY + 1 - len(pending))
            if not block or len(pending) + len(block) > MAX_REPLY:
                raise InstallError(UNAVAILABLE)
            pending.extend(block)
        raw, rest = pending.split(b'\n', 1)
        pending[:] = rest
        check()
        return json.loads(raw)

    def write(process, data):
        check()
        process.stdin.write(json.dumps(data, ensure_ascii=False).encode() + b'\n')
        process.stdin.flush()

    try:
        check()
        with subprocess.Popen([NODE, bundle()], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              stderr=subprocess.DEVNULL, env={'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}) as process:
            try:
                with selectors.DefaultSelector() as watch:
                    watch.register(process.stdout, selectors.EVENT_READ)
                    write(process, {'pool': identity.issuer.rsplit('/', 1)[-1], 'client': identity.client_id,
                                    'username': username, 'value': password})
                    del password
                    while True:
                        reply = read(process, watch)
                        if isinstance(reply, dict) and set(reply) == {'error'} and isinstance(reply['error'], str):
                            raise InstallError(ERRORS.get(reply['error'], UNAVAILABLE))
                        if isinstance(reply, dict) and set(reply) == {'access_token'}:
                            token = reply['access_token']
                            if isinstance(token, str) and re.fullmatch(r'[A-Za-z0-9._-]{1,8192}', token):
                                return token
                            raise InstallError(UNAVAILABLE)
                        if (not isinstance(reply, dict) or set(reply) != {'challenge'}
                                or not isinstance(reply['challenge'], str) or reply['challenge'] not in PROMPTS):
                            raise InstallError(UNAVAILABLE)
                        kind = reply['challenge']
                        answer = request_challenge(kind, PROMPTS[kind], check)
                        if not isinstance(answer, str) or not 1 <= len(answer) <= 256:
                            raise InstallError('Enter the requested verification value.')
                        if kind.endswith('_code') and not re.fullmatch(r'[0-9]{6}', answer):
                            raise InstallError('Enter the six-digit verification code.')
                        write(process, {'value': answer})
                        del answer
            finally:
                if process.poll() is None:
                    process.kill()
    except InstallError:
        raise
    except (OSError, ValueError, RecursionError):
        raise InstallError(UNAVAILABLE) from None
