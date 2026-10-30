-- One nonrepeatable claim per character/definition. Advertise only after COMMIT;
-- consumption and the complete item-state/receipt update commit atomically.
CREATE TABLE theia.reward_claims (
    realm_id UUID NOT NULL,
    claim_id UUID NOT NULL CHECK (claim_id <> '00000000-0000-0000-0000-000000000000'),
    character_id UUID NOT NULL,
    reward_id TEXT NOT NULL CHECK (octet_length(reward_id) BETWEEN 1 AND 96),
    world_id BYTEA NOT NULL CHECK (octet_length(world_id) = 32 AND world_id <> decode(repeat('00',32),'hex')),
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash) = 32 AND profile_hash <> decode(repeat('00',32),'hex')),
    consumed_revision BIGINT CHECK (consumed_revision > 0),
    consumed_command BYTEA CHECK (octet_length(consumed_command) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id, claim_id),
    UNIQUE (realm_id, character_id, reward_id),
    FOREIGN KEY (realm_id, character_id) REFERENCES theia.characters(realm_id, character_id),
    CHECK ((consumed_revision IS NULL) = (consumed_command IS NULL))
);
