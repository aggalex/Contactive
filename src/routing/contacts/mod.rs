use rocket::{State};
use rocket_contrib::json::Json;
use crate::{db::{DBState, QueryById, Register, contact::{Contact, NewContact, UserContactRelation}, user::{IsUser, User}}, verification::jwt::LoginHandler};
use super::{JsonResponse, StatusCatch, Verifier};
use crate::routing::{ToJson, EmptyResponse};
use crate::verification::jwt::Token;
use crate::db::{Delete, Update};
use crate::db::contact::UpdateContact;

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

#[delete("/contacts/<id>")]
pub fn delete_contact (db: State<DBState>, jwt_key: State<LoginHandler>, id: i64, token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    Contact::delete(&**db, id).to_status()?;
    
    Ok(())
}

#[patch("/contacts/<id>", format = "application/json", data = "<contact>")]
pub fn edit_contact (db: State<DBState>, jwt_key: State<LoginHandler>, id: i64, contact: Json<UpdateContact>, token: Token) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    contact.update(&**db, id).to_status()?.to_json()
}
