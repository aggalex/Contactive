use diesel::QueryDsl;
use rocket::{State, http::Cookies};
use rocket_contrib::json::Json;
use crate::{db::{DBState, Register, contact::{Contact, NewContact, UserContactRelation}, schema::{contacts, users_contacts_join}}, jwt::JwtState};
use crate::diesel::*;
pub mod personas;

#[derive(Responder)]
pub enum ContactsResponse {
    #[response(status = 500, content_type = "plain")]
    InternalServerError(String),
    #[response(status = 401, content_type = "plain")]
    Unauthorized(()),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 200, content_type = "json")]
    Success(String)
}

#[get("/contacts")]
pub fn get_contacts (db: State<DBState>, jwt_key: State<JwtState>, cookies: Cookies) -> ContactsResponse {
    let user = if let Ok(user) = (*jwt_key).verify (cookies) { user } else {
        return ContactsResponse::Unauthorized(())
    };

    let reply = if let Ok(c) = users_contacts_join::table
        .filter (users_contacts_join::user_id.eq(user.custom.user_id))
        .inner_join(contacts::table)
        .load::<((i64, i64), Contact)> (&**db) 
    {
        let strings = c.into_iter()
            .map(|entry| serde_json::to_string(&entry.1))
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap())
            .collect::<Vec<String>>();
        
        if let Ok(reply) = serde_json::to_string(&strings) { reply } else {
            return ContactsResponse::InternalServerError("Serialization failed".to_string ())
        }
    } else {
        return ContactsResponse::InternalServerError("Query failed".to_string ())
    };

    ContactsResponse::Success(reply)
}

#[post("/personas", format = "application/json", data = "<contacts>")]
pub fn add_contacts (db: State<DBState>, jwt_key: State<JwtState>, contacts: Json<Vec<NewContact>>, cookies: Cookies) -> ContactsResponse {
    let user = if let Ok(user) = (*jwt_key).verify (cookies) { user } else {
        return ContactsResponse::Unauthorized(())
    };

    let results = (&*contacts)
            .into_iter ()
            .map(|contact| contact.clone ()
                    .register (&*db) 
                    .and_then (|contact|
                        UserContactRelation(user.custom.user_id, contact.id)
                                .register (&*db)
                                .map(|_| contact)
                    ))
            .collect::<Vec<Result<Contact, diesel::result::Error>>> ();

    if (&results).into_iter()
                    .map(|result| match result {
                        Ok(_) => None,
                        Err(err) => Some(err)
                    })
                    .filter(|result| result.is_some())
                    .take(1)
                    .count() > 1 
    {
        return ContactsResponse::InternalServerError("Failed to insert contacts to database".to_string ());
    }

    let contacts = results.into_iter ()
        .map(|contact| contact.unwrap ())
        .collect::<Vec<Contact>> ();

    let response = if let Ok(str) = serde_json::to_string(&contacts) { str } else {
        return ContactsResponse::InternalServerError("Failed to serialize result".to_string ());
    };

    ContactsResponse::Success(response)
}