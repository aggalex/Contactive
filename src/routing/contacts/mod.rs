use diesel::QueryDsl;
use rocket::{State, http::{Cookies, Status}, response::Responder};
use rocket_contrib::json::Json;
use crate::{db::{DBState, Register, contact::{Contact, NewContact, UserContactRelation}, schema::{contacts, users_contacts_join}}, verification::jwt::DefaultJwtHandler};
use crate::diesel::*;
use super::{JsonResponse, Verifier};
use crate::routing::JsonResponseNew;
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
pub fn get_contacts (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, cookies: Cookies) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (cookies)?;

    let fetched = users_contacts_join::table
        .filter (users_contacts_join::user_id.eq(user.custom.user_id))
        .inner_join(contacts::table)
        .load::<((i64, i64), Contact)> (&**db) 
        .map_err(|_| Status::InternalServerError)?;

    JsonResponse::new(
        &fetched.into_iter()
            .map(|entry| entry.1)
            .collect::<Vec<Contact>>()
    )
}

#[post("/personas", format = "application/json", data = "<contacts>")]
pub fn add_contacts (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, contacts: Json<Vec<NewContact>>, cookies: Cookies) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (cookies)?;

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
        return Err(Status::InternalServerError);
    }

    let contacts = results.into_iter ()
        .map(|contact| contact.unwrap ())
        .collect::<Vec<Contact>> ();

    JsonResponse::new(&contacts)
}