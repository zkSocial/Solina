CREATE TABLE auth_credentials
(   
    id         INTEGER  NOT NULL  PRIMARY KEY AUTOINCREMENT,
    address    TEXT     NOT NULL,
    challenge  TEXT     NOT NULL,
    is_auth    BOOL     NOT NULL,
    is_valid   BOOL     NOT NULL,
    created_at DATETIME NOT NULL
);

CREATE INDEX public_key_base_quote_token ON intents (public_key, base_token, quote_token);
