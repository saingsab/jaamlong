-- Add migration script here
INSERT INTO tbl_networks (network_name, network_rpc, chain_id, decimal_value, created_by)
VALUES
    ('Goerli','https://eth-goerli.public.blastapi.io', 5, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Binance','https://data-seed-prebsc-1-s1.binance.org:8545/', 97, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Fuji', 'https://ava-testnet.public.blastapi.io/ext/bc/C/rpc', 43113, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('Polygon','https://polygon-testnet-rpc.allthatnode.com:8545', 80001, 18, '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0');