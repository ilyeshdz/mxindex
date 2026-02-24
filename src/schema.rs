// @generated automatically by Diesel CLI.

diesel::table! {
    servers (id) {
        id -> Integer,
        domain -> Text,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        logo_url -> Nullable<Text>,
        theme -> Nullable<Text>,
        registration_open -> Nullable<Bool>,
        public_rooms_count -> Nullable<Integer>,
        version -> Nullable<Text>,
        federation_version -> Nullable<Text>,
        delegated_server -> Nullable<Text>,
        room_versions -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
