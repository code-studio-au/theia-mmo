"""Private, bounded account bridge for the persistent native game window.

Only cosmetic choices and public roster/status cross this handoff. OIDC sessions
stay in AccountClient; the existing one-use ticket is published separately.
"""
import json
import os
import time
from account_login import AccountError
from character_appearance import validate


def publish(directory, name, value):
    pending = directory / (name + '.pending')
    with os.fdopen(os.open(pending, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), 'w') as file:
        json.dump(value, file)
    pending.replace(directory / name)


def run(account, directory, ticket, stopped):
    from native_login import Credentials
    credentials = Credentials(directory)
    try:
        return _run(account, directory, ticket, stopped, credentials)
    finally:
        credentials.close()


def _run(account, directory, ticket, stopped, credentials):
    phase, characters, serial = 'login', [], 0

    def state(message='', busy=False):
        publish(directory, 'frontend-state.json', dict(schema=1, serial=serial,
                phase=phase, busy=busy, message=message, characters=characters,
                registration=os.environ.get('THEIA_NATIVE_REGISTRATION') == '1'))

    state('Sign in to choose your adventurer.')
    while not stopped():
        command = directory / 'frontend-command.json'
        data = credentials.receive()
        native_credentials = data is not None
        if data is None:
            try:
                with command.open('rb') as file:
                    raw = file.read(2049)
            except FileNotFoundError:
                time.sleep(.05)
                continue
            command.unlink()
            try:
                if len(raw) > 2048:
                    raise ValueError()
                data = json.loads(raw)
            except ValueError:
                state('Invalid menu command')
                continue
        try:
            if not isinstance(data, dict) or type(data.get('serial')) is not int or data['serial'] != serial + 1:
                raise ValueError('Stale or invalid menu command')
            serial = data['serial']
            action = data.get('action')
            fields = {'serial', 'action'}
            if native_credentials and action in ('credentials', 'register') and phase == 'login' and set(data) == fields | {'username', 'password'} | ({'email'} if action == 'register' else set()):
                from native_login import authorize
                state('Creating account…' if action == 'register' else 'Signing in…', busy=True)
                try:
                    account.login(lambda url: authorize(url, account._base, data['username'], data['password'], **({'email': data['email']} if action == 'register' else {})))
                finally:
                    data.clear()
                state('Retrieving character list…', busy=True)
                characters = account.characters()
                phase = 'characters'
                state('Choose your adventurer.' if characters else 'Create your first adventurer.')
            elif action == 'logout' and phase == 'characters' and set(data) == fields:
                state('Signing out…', busy=True)
                account.logout()
                characters = []
                phase = 'login'
                state()
            elif action == 'create' and phase == 'characters' and set(data) == fields | {'name', 'appearance'}:
                if len(characters) >= 8:
                    raise ValueError('Your account already has eight characters.')
                account.create_character(data['name'], validate(data['appearance']))
                characters = account.characters()
                state('Character saved. Select your adventurer and enter the world.')
            elif action == 'enter' and phase == 'characters' and set(data) == fields | {'id'}:
                if data['id'] not in {row['id'] for row in characters}:
                    raise ValueError('Choose a character from this account.')
                state('Preparing your world connection…', busy=True)
                address = account.enter(data['id'], ticket)
                publish(directory, 'frontend-admission.json', {'address': address})
                phase = 'entering'
                state('Entering the world…')
                return
            else:
                raise ValueError('This action is unavailable on the current screen.')
        except (ValueError, AccountError) as error:
            state(str(error))
    raise KeyboardInterrupt
