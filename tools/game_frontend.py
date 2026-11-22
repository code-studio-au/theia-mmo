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
    phase, characters, serial = 'login', [], 0

    def state(message='', busy=False):
        publish(directory, 'frontend-state.json', dict(schema=1, serial=serial,
                phase=phase, busy=busy, message=message, characters=characters))

    state('Sign in to choose your adventurer.')
    while not stopped():
        command = directory / 'frontend-command.json'
        try:
            with command.open('rb') as file:
                raw = file.read(2049)
        except FileNotFoundError:
            time.sleep(.05)
            continue
        command.unlink()
        try:
            if len(raw) > 2048:
                raise ValueError('Command exceeds size limit')
            data = json.loads(raw)
            if not isinstance(data, dict) or type(data.get('serial')) is not int or data['serial'] != serial + 1:
                raise ValueError('Stale or invalid menu command')
            serial = data['serial']
            action = data.get('action')
            fields = {'serial', 'action'}
            if action == 'login' and phase == 'login' and set(data) == fields:
                state('Complete sign-in in your browser, then return here.', busy=True)
                account.login()
                characters = account.characters()
                phase = 'characters'
                state('Choose an adventurer or create a new character.' if characters else 'No characters yet. Create your first adventurer.')
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
