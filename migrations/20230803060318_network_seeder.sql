-- Add migration script here
INSERT INTO tbl_networks (network_name, network_rpc, bridge_address, bridge_fee, chain_id, decimal_value, created_by)
VALUES
    ('Goerli','https://eth-goerli.public.blastapi.io', '0xCF6F0d155989B11Ba3882e99c72f609f0C06e086', 0.005, 5, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Binance','https://data-seed-prebsc-1-s1.binance.org:8545/', '0xCF6F0d155989B11Ba3882e99c72f609f0C06e086', 0.005, 97, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Fuji', 'https://ava-testnet.public.blastapi.io/ext/bc/C/rpc', '0xCF6F0d155989B11Ba3882e99c72f609f0C06e086', 0.005, 43113, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Polygon','https://polygon-testnet-rpc.allthatnode.com:8545', '0xCF6F0d155989B11Ba3882e99c72f609f0C06e086', 0.005, 80001, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0');