-- Add migration script here
INSERT INTO tbl_status (status_name, created_by)
VALUES
    ('pending', '4e4adca8-6c6b-4a3f-92c2-8be6d4571ed0'),
    ('complete', 'a5489dd8-74cc-4f1a-a5e3-d7f34821f7c7'),
    ('failed', 'b47a4b6c-ff0f-4020-bd1e-75f36d35b27e');