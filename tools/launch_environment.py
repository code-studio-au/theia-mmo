"""Keep local account/database configuration out of native client processes."""


def client_environment(source):
    """Copy OS/game settings without server-service or owned-test configuration.

    Libpq also accepts password files/service files, not just a database URL.
    Keep the parent's environment intact: the server and browser still need it.
    Match case-insensitively for Windows environment handling.
    """
    def retained(key):
        name = key.upper()
        return (name != 'DATABASE_URL'
                and not name.startswith(('PG', 'THEIA_DATABASE_', 'THEIA_OIDC_',
                                         'THEIA_ACCOUNT_', 'THEIA_TEST_')))
    return {key: value for key, value in source.items() if retained(key)}
