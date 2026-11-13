"""Bounded character-selection handoff inside a private owned launch directory."""
import json
import os
import time


def select_character(account, directory, stopped, timeout=120):
    if not 0 <= timeout <= 120:
        raise ValueError('Character selection timeout must be within 0..120 seconds')
    if stopped():
        raise KeyboardInterrupt
    characters = account.characters()
    pending = directory/'characters.pending'
    with os.fdopen(os.open(pending, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'w') as file:
        json.dump(dict(schema=1, characters=characters), file)
    pending.replace(directory/'characters.json')
    if not characters:
        raise ValueError('No characters available for this account')
    allowed = {row['id'] for row in characters}
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if stopped():
            raise KeyboardInterrupt
        try:
            with (directory/'selection.json').open('rb') as file:
                raw = file.read(1025)
        except FileNotFoundError:
            time.sleep(.05)
            continue
        try:
            choice = json.loads(raw) if len(raw) <= 1024 else None
            if not isinstance(choice, dict) or set(choice) != {'schema', 'character_id'} or type(choice['schema']) is not int or choice['schema'] != 1 or not isinstance(choice['character_id'], str) or choice['character_id'] not in allowed:
                raise ValueError()
        except (ValueError, UnicodeDecodeError):
            raise ValueError('Invalid account character selection') from None
        if stopped():
            raise KeyboardInterrupt
        return choice['character_id']
    raise TimeoutError('Character selection timed out')
