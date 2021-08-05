use rocket::{State};
use rocket_contrib::json::Json;
use crate::{db::{DBState, QueryById, Register, contact::{Contact, NewContact, UserContactRelation}, user::{IsUser, User}}, verification::jwt::LoginHandler};
use super::{JsonResponse, StatusCatch, Verifier};
use crate::routing::{ToJson, EmptyResponse};
use crate::verification::jwt::Token;
use crate::db::{Delete, Update};
use crate::db::contact::UpdateContact;
use crate::db::user::{UserId, ForUser};

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
pub fn delete_contact (db: State<DBState>, id: i64, user: UserId) -> EmptyResponse {

    ForUser::<Contact>::from(user).delete(&**db, id).to_status()?;

    Ok(())
}

#[patch("/contacts/<id>", format = "application/json", data = "<contact>")]
pub fn edit_contact (db: State<DBState>, id: i64, contact: Json<UpdateContact>, user: UserId) -> JsonResponse {
    let factory: ForUser<UpdateContact> = user.into();
    factory.get(contact.into_inner())
        .update(&**db, id)
        .to_status()?
        .to_json()
}


