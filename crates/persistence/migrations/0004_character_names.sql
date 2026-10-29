-- Existing unnamed development characters remain intact. Newly created account
-- characters receive immutable IDs and realm-scoped case-insensitive names.
CREATE TABLE theia.character_names (
    realm_id UUID NOT NULL,
    character_id UUID NOT NULL,
    display_name TEXT NOT NULL CHECK (
        octet_length(display_name) BETWEEN 3 AND 20
        AND display_name COLLATE "C" ~ '^[A-Za-z]([A-Za-z]|[''-][A-Za-z])+$'
    ),
    name_key TEXT GENERATED ALWAYS AS (lower(display_name COLLATE "C")) STORED,
    PRIMARY KEY (realm_id, character_id),
    FOREIGN KEY (realm_id, character_id) REFERENCES theia.characters(realm_id, character_id),
    CONSTRAINT character_names_realm_name_unique UNIQUE (realm_id, name_key)
);
