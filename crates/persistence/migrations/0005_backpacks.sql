-- Inventory is independent of superseding ordinary pose checkpoints. A bounded
-- latest-command receipt recovers a lost success response; older revisions never
-- reapply. Catalog/state are validated by the checked authority before use.
CREATE TABLE theia.backpacks (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    catalog_hash BYTEA NOT NULL CHECK (
        octet_length(catalog_hash) = 32 AND catalog_hash <> decode(repeat('00',32),'hex')
    ),
    revision BIGINT NOT NULL CHECK (revision >= 0),
    state BYTEA NOT NULL CHECK (octet_length(state) BETWEEN 1 AND 4096),
    last_command BYTEA CHECK (octet_length(last_command) = 32),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id, character_id),
    FOREIGN KEY (realm_id, character_id) REFERENCES theia.characters(realm_id, character_id),
    CHECK ((revision = 0 AND last_command IS NULL) OR (revision > 0 AND last_command IS NOT NULL))
);
