use rocket::{State, http::{Cookies, Status}};
use rocket_contrib::json::Json;
use crate::{db::{DBState, Register, contact::{Contact, NewContact, UserContactRelation}, user::{IsUser, User}}, verification::jwt::DefaultJwtHandler};
use super::{JsonResponse, Verifier};
pub mod personas;
use crate::routing::ToJson;

#[get("/contacts")]
pub fn get_contacts (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, cookies: Cookies) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (cookies)?;

    User::query_by_id (user.custom.user_id, &**db)
        .and_then (|user| user.get_contacts(&**db)) 
        .map_err(|_| Status::InternalServerError)?
        .to_json()
}

#[post("/personas", format = "application/json", data = "<contacts>")]
pub fn add_contacts (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, contacts: Json<Vec<NewContact>>, cookies: Cookies) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (cookies)?;

    (&*contacts)
            .into_iter ()
            .map(|contact| {
                    let contact = contact.clone ()
                            .register (&*db)?;
                    UserContactRelation(
                            user.custom.user_id, 
                            contact.id
                    ).register (&*db)?;
                    Ok(contact)
            })
            .collect::<Result<Vec<Contact>, diesel::result::Error>> ()
            .map_err (|_| Status::InternalServerError)?
            .to_json ()
}