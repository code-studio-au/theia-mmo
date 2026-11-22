-- Cosmetic catalogue v1: body, face, hair, facial hair, skin, hair colour, outfit.
-- Existing records receive the default hero; ownership/world/position are untouched.
ALTER TABLE theia.characters
    ADD COLUMN appearance BYTEA NOT NULL DEFAULT decode('00000100000000', 'hex'),
    ADD CONSTRAINT character_appearance_v1 CHECK (
        octet_length(appearance) = 7
        AND get_byte(appearance,0) < 2 AND get_byte(appearance,1) < 4
        AND get_byte(appearance,2) < 5 AND get_byte(appearance,3) < 4
        AND get_byte(appearance,4) < 4 AND get_byte(appearance,5) < 4
        AND get_byte(appearance,6) < 2
        AND (get_byte(appearance,0) = 0 OR get_byte(appearance,3) = 0)
    );
