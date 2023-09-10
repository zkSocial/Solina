table! {
    intents (id) {
        id -> diesel::sql_types::Integer,
        structured_hash -> Text,
        public_key -> Text,
        signature -> Text,
        base_token -> Text,
        quote_token -> Text,
        quote_amount -> BigInt,
        direction -> Bool,
        min_base_token_amount -> BigInt,
        created_at -> Timestamp,
    }
}

table! {
    auth_credentials(id) {
        id -> diesel::sql_types::Integer,
        address -> Text,
        challenge -> Text,
        is_auth -> Bool,
        is_valid -> Bool,
        created_at -> Timestamp,
    }
}
