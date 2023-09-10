CREATE TABLE auth_credentials
(   
    id         INTEGER  NOT NULL  PRIMARY KEY AUTOINCREMENT,
    address    TEXT     NOT NULL,
    challenge  TEXT     NOT NULL,
    is_auth    BOOL     NOT NULL,
    is_valid   BOOL     NOT NULL,
    created_at DATETIME NOT NULL
);
