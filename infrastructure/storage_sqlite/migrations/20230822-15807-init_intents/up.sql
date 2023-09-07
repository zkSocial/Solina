CREATE TABLE intents
(   
    id                    INTEGER  NOT NULL  PRIMARY KEY,
    structured_hash       TEXT     NOT NULL,
    public_key            TEXT     NOT NULL,
    signature             TEXT     NOT NULL,
    base_token            TEXT     NOT NULL,
    quote_token           TEXT     NOT NULL,
    quote_amount          BIGINT   NOT NULL,
    direction             BOOLEAN  NOT NULL,
    min_base_token_amount BIGINT   NOT NULL,
    created_at            DATETIME NOT NULL
);

CREATE INDEX public_key_base_quote_token ON intents (public_key, base_token, quote_token);
