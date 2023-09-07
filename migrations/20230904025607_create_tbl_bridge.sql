-- Add migration script here
CREATE TABLE
    IF NOT EXISTS tbl_bridge (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        bridge_address VARCHAR(64) NOT NULL,
        bridge_fee FLOAT8 NOT NULL,
        created_by UUID,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );