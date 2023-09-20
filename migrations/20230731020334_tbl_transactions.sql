-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE
    IF NOT EXISTS tbl_transactions (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        sender_address VARCHAR(64) NOT NULL,
        receiver_address VARCHAR(64) NOT NULL,
        origin_network UUID,
        destin_network UUID,
        from_token_address VARCHAR(64) NOT NULL,
        to_token_address VARCHAR(64) NOT NULL,
        transfer_amount NUMERIC,
        bridge_fee NUMERIC,
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