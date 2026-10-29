-- Explicit fixture issuer binding. This is not an external account provider.
CREATE TABLE theia.development_bindings (
    realm_id UUID NOT NULL CHECK (realm_id <> '00000000-0000-0000-0000-000000000000'),
    slot SMALLINT NOT NULL CHECK (slot BETWEEN 1 AND 8),
    character_id UUID NOT NULL CHECK (character_id <> '00000000-0000-0000-0000-000000000000'),
    owner_id UUID NOT NULL CHECK (owner_id <> '00000000-0000-0000-0000-000000000000'),
    PRIMARY KEY (realm_id,slot),
    UNIQUE (realm_id,character_id)
);
