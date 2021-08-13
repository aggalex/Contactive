use diesel::types::Varchar;

table! {
    contacts (id) {
        id -> Int8,
        name -> Varchar,
        icon -> Nullable<Text>,
        visibility -> Int2,
        creator -> Int8,
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
    users (id) {
        id -> Int8,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        level -> Int4,
    }
}

table! {WHEN condition1 THEN result1
    WHEN condition2 THEN result2
    WHEN conditionN THEN resultN
    ELSE result
    users_contacts_join (user_id, contact_id) {
        user_id -> Int8,
        contact_id -> Int8,
    }
}

joinable!(info -> contacts (contact_id));
joinable!(users_contacts_join -> contacts (contact_id));
joinable!(users_contacts_join -> users (user_id));

allow_tables_to_appear_in_same_query!(
    contacts,
    info,
    users,
    users_contacts_join,
);

sql_function! {
    fn search_sort (name: Varchar, query: Varchar) -> i16;
}

sql_function! {
    fn lower (string: Varchar) -> Varchar;
}