-- Initial generation rights are separate from the random ticket and later need.
-- Existing ordinary rolls stay eligible and have no quest-state audit revisions.
ALTER TABLE theia.loot_rolls
    ADD COLUMN generation_eligible BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN generation_bag_revision BIGINT,
    ADD COLUMN generation_journal_revision BIGINT,
    ADD CONSTRAINT loot_generation_snapshot CHECK (
        (generation_bag_revision IS NULL AND generation_journal_revision IS NULL AND generation_eligible)
        OR (generation_bag_revision >= 0 AND generation_journal_revision >= 0
            AND generation_bag_revision IS NOT NULL AND generation_journal_revision IS NOT NULL)
    );
