-- One opt-in development objective per character. Positive victim-life receipts
-- and progress commit atomically with enabled XP; no client supplies counters.
CREATE TABLE theia.quest_objectives (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    revision BIGINT NOT NULL CHECK (revision>=0),
    kill_count INTEGER NOT NULL CHECK (kill_count BETWEEN 0 AND 100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id,character_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.characters(realm_id,character_id)
);
CREATE TABLE theia.quest_kill_receipts (
    realm_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    event_id BYTEA NOT NULL CHECK (octet_length(event_id)=16 AND substring(event_id FROM 1 FOR 8)<>decode(repeat('00',8),'hex') AND substring(event_id FROM 9 FOR 8)<>decode(repeat('00',8),'hex')),
    character_id UUID NOT NULL,
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    creature_id TEXT NOT NULL CHECK (octet_length(creature_id) BETWEEN 1 AND 96),
    revision BIGINT NOT NULL CHECK (revision>0),
    kill_count INTEGER NOT NULL CHECK (kill_count BETWEEN 1 AND 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id,world_id,event_id),
    UNIQUE (realm_id,character_id,revision),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_objectives(realm_id,character_id)
);
