CREATE UNLOGGED TABLE order_details (
    id        SERIAL NOT NULL PRIMARY KEY,
    location  TEXT NOT NULL,
    timestamp TIMESTAMP WITH time zone NOT NULL,
    signature TEXT NOT NULL,
    material  INTEGER NOT NULL,
    a         INTEGER NOT NULL,
    b         INTEGER NOT NULL,
    c         INTEGER NOT NULL,
    d         INTEGER NOT NULL
) WITH (autovacuum_enabled=false); 