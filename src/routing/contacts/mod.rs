use rocket::{State};
use rocket_contrib::json::Json;
use crate::db::{DBState, QueryById, Register, contact::Contact, user::{IsUser, User}};
use super::{JsonResponse, StatusCatch};
use crate::routing::{ToJson, EmptyResponse};
use crate::db::{Delete, Update};
use crate::db::contact::{UpdateContact, PostContact};
use crate::db::user::{UserId, ForUser};

pub mod info;

#[get("/contacts")]
pub fn get_contacts (db: State<DBState>, user: UserId) -> JsonResponse {
    User::query_by_id (*user, &db)
        .and_then (|user| user.get_contacts(&db)) 
        .to_status()?
        .to_json()
}

#[post("/contacts", format = "application/json", data = "<contacts>")]
pub fn add_contacts (db: State<DBState>, contacts: Json<Vec<PostContact>>, user: UserId) -> JsonResponse {
    let factory = ForUser::<PostContact>::from(user);
    contacts.into_inner()
        .into_iter ()
        .map(|contact|
            factory.relate(contact)
                .register (&db))
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

#[get("/contacts/public?<q>&<page>&<buffer>")]
pub fn search_public_contacts (db: State<DBState>, q: String, page: u32, buffer: Option<u32>) -> JsonResponse {
    let buffer = buffer.unwrap_or(10);
    Contact::search_public(&**db, page as i64, buffer as i64, q)
        .to_status()?
        .to_json()
}