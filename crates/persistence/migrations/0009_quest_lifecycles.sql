-- Explicit acceptance is separate from the retained automatic objective fixture.
-- State is bounded to one nonrepeatable quest per character in this first slice.
CREATE TABLE theia.quest_lifecycles (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    revision BIGINT NOT NULL CHECK (revision BETWEEN 0 AND 102),
    status SMALLINT NOT NULL CHECK (status BETWEEN 0 AND 2),
    kill_count INTEGER NOT NULL CHECK (kill_count BETWEEN 0 AND 100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (realm_id,character_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.characters(realm_id,character_id),
    CHECK ((status=0 AND revision=0 AND kill_count=0)
        OR (status=1 AND revision=kill_count+1)
        OR (status=2 AND kill_count>0 AND revision=kill_count+2))
);
-- At most two action receipts per character. Acceptance retries remain recoverable
-- after later kills/turn-in; only a durable receipt can authorize replay recovery.
CREATE TABLE theia.quest_action_receipts (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    expected_revision BIGINT NOT NULL CHECK (expected_revision BETWEEN 0 AND 101),
    action SMALLINT NOT NULL CHECK (action IN (1,2)),
    command_hash BYTEA NOT NULL CHECK (octet_length(command_hash)=32),
    PRIMARY KEY (realm_id,character_id,expected_revision),
    UNIQUE (realm_id,character_id,action),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_lifecycles(realm_id,character_id),
    CHECK ((action=1 AND expected_revision=0) OR (action=2 AND expected_revision>=2))
);
-- Record qualifying pre-accept/capped no-ops too: retrying an old trusted fact
-- after acceptance must never convert it into new objective progress.
CREATE TABLE theia.quest_lifecycle_kills (
    realm_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32 AND world_id<>decode(repeat('00',32),'hex')),
    event_id BYTEA NOT NULL CHECK (octet_length(event_id)=16 AND substring(event_id FROM 1 FOR 8)<>decode(repeat('00',8),'hex') AND substring(event_id FROM 9 FOR 8)<>decode(repeat('00',8),'hex')),
    character_id UUID NOT NULL,
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash)=32 AND profile_hash<>decode(repeat('00',32),'hex')),
    creature_id TEXT NOT NULL CHECK (octet_length(creature_id) BETWEEN 1 AND 96),
    revision BIGINT NOT NULL CHECK (revision BETWEEN 0 AND 102),
    kill_count INTEGER NOT NULL CHECK (kill_count BETWEEN 0 AND 100),
    counted BOOLEAN NOT NULL,
    PRIMARY KEY (realm_id,world_id,event_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_lifecycles(realm_id,character_id),
    CHECK (NOT counted OR (kill_count>0 AND revision=kill_count+1))
);
CREATE UNIQUE INDEX quest_lifecycle_counted_revision
    ON theia.quest_lifecycle_kills(realm_id,character_id,revision) WHERE counted;
