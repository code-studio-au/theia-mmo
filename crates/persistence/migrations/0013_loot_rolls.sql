-- Persist misses as well as wins: opening/retrying/rejoining cannot reroll a life.
-- Group members share one ticket. Claims and their ticket commit together.
CREATE TABLE theia.loot_rolls (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    roll_key TEXT NOT NULL CHECK (octet_length(roll_key) BETWEEN 1 AND 128),
    victim_life BYTEA NOT NULL CHECK (octet_length(victim_life) = 16
        AND substring(victim_life FROM 1 FOR 8) <> decode(repeat('00',8),'hex')
        AND substring(victim_life FROM 9 FOR 8) <> decode(repeat('00',8),'hex')),
    world_id BYTEA NOT NULL CHECK (octet_length(world_id) = 32),
    profile_hash BYTEA NOT NULL CHECK (octet_length(profile_hash) = 32),
    ticket INTEGER NOT NULL CHECK (ticket BETWEEN 0 AND 9999),
    corpse_x DOUBLE PRECISION NOT NULL CHECK (corpse_x BETWEEN -1.0e12 AND 1.0e12),
    corpse_y DOUBLE PRECISION NOT NULL CHECK (corpse_y BETWEEN -1.0e12 AND 1.0e12),
    corpse_z DOUBLE PRECISION NOT NULL CHECK (corpse_z BETWEEN -1.0e12 AND 1.0e12),
    PRIMARY KEY (realm_id, character_id, roll_key, victim_life),
    FOREIGN KEY (realm_id, character_id) REFERENCES theia.characters(realm_id, character_id)
);
