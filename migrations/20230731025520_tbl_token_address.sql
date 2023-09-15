-- Add migration script here
CREATE TABLE
    IF NOT EXISTS tbl_token_address (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        token_address VARCHAR(64) NOT NULL,
        token_symbol VARCHAR(10) NOT NULL,
        asset_type VARCHAR(10) NOT NULL,
        abi JSONB,
        created_by UUID,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );