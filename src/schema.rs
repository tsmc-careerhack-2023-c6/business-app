// @generated automatically by Diesel CLI.

diesel::table! {
    order_details (id) {
        id -> Int4,
        location -> Text,
        timestamp -> Timestamptz,
        signature -> Text,
        material -> Int4,
        a -> Int4,
        b -> Int4,
        c -> Int4,
        d -> Int4,
    }
}
