-- Legacy journal-only action receipts stay all-NULL. Rewarded mode binds a
-- different quest digest and records the committed XP snapshot for recovery.
ALTER TABLE theia.quest_action_receipts
    ADD COLUMN xp_profile_hash BYTEA,
    ADD COLUMN xp_granted INTEGER,
    ADD COLUMN xp_revision BIGINT,
    ADD COLUMN xp_total INTEGER,
    ADD CONSTRAINT quest_action_xp_metadata CHECK (
        (xp_profile_hash IS NULL AND xp_granted IS NULL AND xp_revision IS NULL AND xp_total IS NULL)
        OR
        (xp_profile_hash IS NOT NULL AND xp_granted IS NOT NULL AND xp_revision IS NOT NULL AND xp_total IS NOT NULL
         AND octet_length(xp_profile_hash)=32 AND xp_profile_hash<>decode(repeat('00',32),'hex')
         AND xp_granted BETWEEN 0 AND 1000 AND xp_revision>=0 AND xp_total BETWEEN 0 AND 10000
         AND xp_granted<=xp_total AND (action=2 OR xp_granted=0)
         AND (xp_granted=0 OR xp_revision>0))
    );
