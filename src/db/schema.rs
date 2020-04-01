table! {
    pdevice (id) {
        id -> Int4,
        devicetypeid -> Int4,
        token -> Varchar,
    }
}

table! {
    pdevicetype (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    puser (id) {
        id -> Int4,
        userid -> Varchar,
        deviceid -> Int4,
    }
}

joinable!(pdevice -> pdevicetype (devicetypeid));
joinable!(puser -> pdevice (deviceid));

allow_tables_to_appear_in_same_query!(pdevice, pdevicetype, puser,);
