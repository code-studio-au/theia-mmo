#!/usr/bin/env python3
"""Owned OIDC/PostgreSQL and two native clients: finite travelling magic."""
import json
from pathlib import Path
from qualify_player_melee import argument_parser, run

if __name__ == '__main__':
    args = vars(argument_parser(__doc__).parse_args())
    args = {key: value.resolve() if isinstance(value, Path) else value for key, value in args.items()}
    result = run(**args, projectile=True)
    print(json.dumps({'passed': result['passed'], 'output': str(args['output'])}))
