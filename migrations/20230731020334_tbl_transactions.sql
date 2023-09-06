-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE
    IF NOT EXISTS tbl_transactions (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        sender_address VARCHAR(64) NOT NULL,
        receiver_address VARCHAR(64) NOT NULL,
        from_token_address VARCHAR(64) NOT NULL,
        to_token_address VARCHAR(64) NOT NULL,
        origin_network UUID,
        destin_network UUID,
        to_asset_type  UUID,
        from_asset_type  UUID,
        transfer_amount BIGINT,
        bridge_fee BIGINT,
        tx_status UUID,
        origin_tx_hash VARCHAR(100),
        destin_tx_hash VARCHAR(100),
        created_by UUID,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );