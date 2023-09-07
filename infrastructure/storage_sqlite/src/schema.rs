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
