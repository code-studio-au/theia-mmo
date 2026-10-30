-- Initial capped progression. The authority supplies earned victim-life facts;
-- receipt and positive XP update commit together before any award observation.
CREATE TABLE theia.progression (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    revision BIGINT NOT NULL CHECK (revision>=0),
    experience INTEGER NOT NULL CHECK (experience BETWEEN 0 AND 10000),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id,character_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.characters(realm_id,character_id)
);
CREATE TABLE theia.experience_receipts (
    realm_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    event_id BYTEA NOT NULL CHECK (octet_length(event_id)=16 AND substring(event_id FROM 1 FOR 8)<>decode(repeat('00',8),'hex') AND substring(event_id FROM 9 FOR 8)<>decode(repeat('00',8),'hex')),
    character_id UUID NOT NULL,
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    creature_id TEXT NOT NULL CHECK (octet_length(creature_id) BETWEEN 1 AND 96),
    granted INTEGER NOT NULL CHECK (granted BETWEEN 1 AND 1000),
    revision BIGINT NOT NULL CHECK (revision>0),
    experience INTEGER NOT NULL CHECK (experience BETWEEN 1 AND 10000 AND experience>=granted),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id,world_id,event_id),
    UNIQUE (realm_id,character_id,revision),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.progression(realm_id,character_id)
);
