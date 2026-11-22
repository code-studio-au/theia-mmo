"""Strict v1 cosmetic catalogue shared by the private account-launcher handoff."""
FIELDS = ('body', 'face', 'hair', 'facial_hair', 'skin', 'hair_colour', 'outfit')
COUNTS = (2, 4, 5, 4, 4, 4, 2)


def validate(value):
    if not isinstance(value, dict) or set(value) != {'schema', *FIELDS}:
        raise ValueError('Invalid character appearance')
    if type(value['schema']) is not int or value['schema'] != 1:
        raise ValueError('Unsupported character appearance')
    if any(type(value[k]) is not int or not 0 <= value[k] < n for k, n in zip(FIELDS, COUNTS)):
        raise ValueError('Invalid character appearance choice')
    if value['body'] == 1 and value['facial_hair'] != 0:
        raise ValueError('Unsupported facial hair for this body')
    return dict(value)
