-- Verified external (issuer, subject) pairs map to server-generated owner IDs.
-- Display names and email addresses never establish ownership.
CREATE TABLE theia.accounts (
    owner_id UUID PRIMARY KEY CHECK (owner_id <> '00000000-0000-0000-0000-000000000000'),
    issuer TEXT NOT NULL CHECK (octet_length(issuer) BETWEEN 1 AND 2048 AND issuer !~ '[[:cntrl:]]'),
    subject TEXT NOT NULL CHECK (octet_length(subject) BETWEEN 1 AND 255 AND subject !~ '[[:cntrl:]]'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (issuer, subject)
);
