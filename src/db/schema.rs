table! {
    contacts (id) {
        id -> Int8,
        name -> Varchar,
        icon -> Nullable<Bytea>,
        visibility -> Int2,
    }
}

table! {
    info (key, value, contact_id) {
        key -> Varchar,
        value -> Varchar,
        contact_id -> Int8,
    }
}

table! {
    personas (id) {
        id -> Int8,
        private -> Bool,
        user_id -> Int8,
    }
}

table! {
    users (id) {
        id -> Int8,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        level -> Int4,
    }
}

table! {
    users_contacts_join (user_id, contact_id) {
        user_id -> Int8,
        contact_id -> Int8,
    }
}

joinable!(contacts -> personas (persona));
joinable!(info -> contacts (contact_id));
joinable!(personas -> users (user_id));
joinable!(users_contacts_join -> contacts (contact_id));
joinable!(users_contacts_join -> users (user_id));

allow_tables_to_appear_in_same_query!(
    contacts,
    info,
    personas,
    users,
    users_contacts_join,
);
