table! {
    addresses (id) {
        id -> Int8,
        street -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        postal_code -> Nullable<Int4>,
        country -> Nullable<Varchar>,
        contact_id -> Int8,
    }
}

table! {
    anniversaries (id) {
        id -> Int8,
        date -> Date,
        contact_id -> Int8,
    }
}

table! {
    contacts (id) {
        id -> Int8,
        name -> Varchar,
        birthday -> Nullable<Date>,
        icon -> Nullable<Bytea>,
        persona -> Nullable<Int8>,
    }
}

table! {
    emails (id) {
        id -> Int8,
        email -> Varchar,
        contact_id -> Int8,
    }
}

table! {
    notes (id) {
        id -> Int8,
        note -> Text,
        contact_id -> Int8,
    }
}

table! {
    personas (id) {
        id -> Int8,
        name -> Varchar,
        private -> Bool,
        user_id -> Int8,
    }
}

table! {
    phones (id) {
        id -> Int8,
        phone -> Varchar,
        contact_id -> Int8,
    }
}

table! {
    social_media (id) {
        id -> Int8,
        link -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        contact_id -> Int8,
    }
}

table! {
    users (id) {
        id -> Int8,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
    }
}

table! {
    users_contacts_join (user_id, contact_id) {
        user_id -> Int8,
        contact_id -> Int8,
    }
}

table! {
    websites (id) {
        id -> Int8,
        link -> Varchar,
        contact_id -> Int8,
    }
}

joinable!(addresses -> contacts (contact_id));
joinable!(anniversaries -> contacts (contact_id));
joinable!(contacts -> personas (persona));
joinable!(emails -> contacts (contact_id));
joinable!(notes -> contacts (contact_id));
joinable!(personas -> users (user_id));
joinable!(phones -> contacts (contact_id));
joinable!(social_media -> contacts (contact_id));
joinable!(users_contacts_join -> contacts (contact_id));
joinable!(users_contacts_join -> users (user_id));
joinable!(websites -> contacts (contact_id));

allow_tables_to_appear_in_same_query!(
    addresses,
    anniversaries,
    contacts,
    emails,
    notes,
    personas,
    phones,
    social_media,
    users,
    users_contacts_join,
    websites,
);
