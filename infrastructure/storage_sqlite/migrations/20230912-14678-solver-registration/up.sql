CREATE TABLE solvers (
    id      INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    address TEXT    NOT NULL
);

CREATE TABLE current_batch_id (
    id INTEGER NOT NULL PRIMARY KEY
);