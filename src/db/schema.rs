table! {
    pdevicetype (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    puser (id) {
        id -> Int4,
        user_id -> Varchar,
        device_type_id -> Int4,
        token -> Varchar,
    }
}

joinable!(puser -> pdevicetype (device_type_id));

allow_tables_to_appear_in_same_query!(pdevicetype, puser,);
