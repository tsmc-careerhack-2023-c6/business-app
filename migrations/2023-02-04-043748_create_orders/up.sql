CREATE TABLE order_details (
    id        SERIAL NOT NULL PRIMARY KEY,
    location  TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    signature TEXT NOT NULL,
    material  INTEGER NOT NULL,
    a         INTEGER NOT NULL,
    b         INTEGER NOT NULL,
    c         INTEGER NOT NULL,
    d         INTEGER NOT NULL
); 