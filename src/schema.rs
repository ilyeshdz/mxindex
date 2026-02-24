// @generated automatically by Diesel CLI.

diesel::table! {
    servers (id) {
        id -> Integer,
        domain -> Text,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        registration_open -> Nullable<Bool>,
        public_rooms_count -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
