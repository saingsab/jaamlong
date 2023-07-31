-- Add migration script here
CREATE TABLE
    IF NOT EXISTS tbl_networks (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        network_name VARCHAR(16) NOT NULL,
        created_by UUID,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );