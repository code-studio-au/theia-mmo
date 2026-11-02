-- Independent multi-quest domain; legacy quest state and receipts are preserved.
CREATE TABLE theia.quest_journal_heads (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    world_id BYTEA NOT NULL CHECK (octet_length(world_id)=32),
    catalog_hash BYTEA NOT NULL CHECK (octet_length(catalog_hash)=32),
    revision BIGINT NOT NULL CHECK (revision>=0),
    state_hash BYTEA NOT NULL CHECK (octet_length(state_hash)=32),
    PRIMARY KEY (realm_id,character_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.characters(realm_id,character_id)
);
CREATE TABLE theia.quest_journal_entries (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    quest_id TEXT NOT NULL CHECK (octet_length(quest_id) BETWEEN 1 AND 96),
    definition_version BIGINT NOT NULL CHECK (definition_version BETWEEN 1 AND 4294967295),
    revision BIGINT NOT NULL CHECK (revision>0),
    status SMALLINT NOT NULL CHECK (status BETWEEN 0 AND 2),
    state BYTEA NOT NULL CHECK (octet_length(state) BETWEEN 1 AND 65536),
    PRIMARY KEY (realm_id,character_id,quest_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_journal_heads(realm_id,character_id)
);
-- One occurrence may count for multiple quests and eligible characters. The
-- stable recipient is part of the receipt identity, unlike the old solo fixture.
CREATE TABLE theia.quest_journal_events (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    event_id BYTEA NOT NULL CHECK (octet_length(event_id)=16 AND event_id<>decode(repeat('00',16),'hex')),
    event_hash BYTEA NOT NULL CHECK (octet_length(event_hash)=32),
    journal_revision BIGINT NOT NULL CHECK (journal_revision>0),
    PRIMARY KEY (realm_id,character_id,event_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_journal_heads(realm_id,character_id)
);
CREATE TABLE theia.quest_journal_actions (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    request_id BYTEA NOT NULL CHECK (octet_length(request_id)=16 AND request_id<>decode(repeat('00',16),'hex')),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash)=32),
    quest_id TEXT NOT NULL CHECK (octet_length(quest_id) BETWEEN 1 AND 96),
    action SMALLINT NOT NULL CHECK (action IN (1,2)),
    journal_revision BIGINT NOT NULL CHECK (journal_revision>0),
    xp_granted INTEGER NOT NULL CHECK (xp_granted BETWEEN 0 AND 1000),
    xp_revision BIGINT CHECK (xp_revision>=0),
    xp_total INTEGER CHECK (xp_total>=0),
    PRIMARY KEY (realm_id,character_id,request_id),
    FOREIGN KEY (realm_id,character_id) REFERENCES theia.quest_journal_heads(realm_id,character_id),
    CHECK ((action=1 AND xp_granted=0 AND xp_revision IS NULL AND xp_total IS NULL)
        OR (action=2 AND xp_revision IS NOT NULL AND xp_total IS NOT NULL))
);
