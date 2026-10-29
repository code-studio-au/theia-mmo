-- Persistent UUIDs never reuse transport IDs or per-world EntityIds.
CREATE TABLE theia.characters (
    realm_id UUID NOT NULL CHECK (realm_id <> '00000000-0000-0000-0000-000000000000'),
    character_id UUID NOT NULL CHECK (character_id <> '00000000-0000-0000-0000-000000000000'),
    owner_id UUID NOT NULL CHECK (owner_id <> '00000000-0000-0000-0000-000000000000'),
    world_id BYTEA NOT NULL CHECK (octet_length(world_id) = 32 AND world_id <> decode(repeat('00',32),'hex')),
    x DOUBLE PRECISION NOT NULL CHECK (x BETWEEN -1.0e12 AND 1.0e12),
    y DOUBLE PRECISION NOT NULL CHECK (y BETWEEN -1.0e12 AND 1.0e12),
    z DOUBLE PRECISION NOT NULL CHECK (z BETWEEN -1.0e12 AND 1.0e12),
    writer_session UUID NOT NULL CHECK (writer_session <> '00000000-0000-0000-0000-000000000000'),
    checkpoint_sequence BIGINT NOT NULL CHECK (checkpoint_sequence >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id, character_id)
);
