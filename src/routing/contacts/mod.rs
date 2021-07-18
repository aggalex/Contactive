use rocket::{State};
use rocket_contrib::json::Json;
use crate::{db::{DBState, QueryById, Register, contact::{Contact, NewContact, UserContactRelation}, user::{IsUser, User}}, verification::jwt::LoginHandler};
use super::{JsonResponse, StatusCatch, Verifier};
use crate::routing::ToJson;
use crate::verification::jwt::Token;

pub mod personas;
pub mod info;

#[get("/contacts")]
pub fn get_contacts (db: State<DBState>, jwt_key: State<LoginHandler>, token: Token) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    User::query_by_id (user.custom.user_id, &db)
        .and_then (|user| user.get_contacts(&db)) 
        .to_status()?
        .to_json()
}

#[post("/contacts", format = "application/json", data = "<contacts>")]
pub fn add_contacts (db: State<DBState>, jwt_key: State<LoginHandler>, contacts: Json<Vec<NewContact>>, token: Token) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    (&*contacts)
        .into_iter ()
        .map(|contact| {
            let contact = contact.clone ()
                .register (&db)?;
            UserContactRelation(
                user.custom.user_id, 
                contact.id
            ).register (&db)?;
            Ok(contact)
        })
        .collect::<Result<Vec<Contact>, diesel::result::Error>> ()
        .to_status()?
        .to_json ()
}