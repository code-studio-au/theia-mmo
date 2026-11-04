-- Existing once-character claims retain their original rows and receipts.
-- A repeatable entitlement is bound to one recipient and actual victim life.
ALTER TABLE theia.reward_claims
    DROP CONSTRAINT reward_claims_realm_id_character_id_reward_id_key,
    ADD COLUMN claim_scope SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN victim_life BYTEA,
    ADD COLUMN corpse_x DOUBLE PRECISION,
    ADD COLUMN corpse_y DOUBLE PRECISION,
    ADD COLUMN corpse_z DOUBLE PRECISION,
    ADD CONSTRAINT reward_claim_source CHECK (
        (claim_scope = 0 AND victim_life IS NULL AND corpse_x IS NULL AND corpse_y IS NULL AND corpse_z IS NULL)
        OR
        (claim_scope = 1 AND victim_life IS NOT NULL AND octet_length(victim_life) = 16
         AND substring(victim_life FROM 1 FOR 8) <> decode(repeat('00',8),'hex')
         AND substring(victim_life FROM 9 FOR 8) <> decode(repeat('00',8),'hex')
         AND corpse_x IS NOT NULL AND corpse_x BETWEEN -1.0e12 AND 1.0e12
         AND corpse_y IS NOT NULL AND corpse_y BETWEEN -1.0e12 AND 1.0e12
         AND corpse_z IS NOT NULL AND corpse_z BETWEEN -1.0e12 AND 1.0e12)
    );
CREATE UNIQUE INDEX reward_once_character ON theia.reward_claims(realm_id,character_id,reward_id) WHERE claim_scope = 0;
CREATE UNIQUE INDEX reward_per_victim_life ON theia.reward_claims(realm_id,character_id,reward_id,victim_life) WHERE claim_scope = 1;
CREATE INDEX reward_pending_life_page ON theia.reward_claims(realm_id,character_id,created_at,claim_id) WHERE claim_scope = 1 AND consumed_revision IS NULL;
