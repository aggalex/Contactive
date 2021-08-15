use rocket::{State};
use rocket_contrib::json::Json;
use crate::db::{DBState, QueryById, Register, contact::Contact, user::{IsUser, User}};
use super::{JsonResponse, StatusCatch};
use crate::routing::{ToJson, EmptyResponse};
use crate::db::{Delete, Update};
use crate::db::contact::{UpdateContact, PostContact, UserContactRelation, IsContact, Visibility, Entitlement, Validate};
use crate::db::user::{UserId, ForUser};
use crate::verification::jwt::contact_jwt::{ContactJwtHandler, ContactJwt};
use crate::verification::jwt::{JwtHandler, Jwt};
use rocket::http::Status;
use serde::{Serialize, Deserialize};
use crate::db::contact::info::BareInfo;

pub mod info;

#[get("/contacts/<id>")]
pub fn get_contact_by_id (db: State<DBState>, id: i64, user: UserId) -> JsonResponse {
    ForUser::<Contact>::from(user).query_by_id(id, &**db)
        .to_status()?
        .to_json()
}

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
    if (&*contacts).into_iter()
        .filter(|contact| !contact.validate())
        .take(1)
        .collect::<Vec<&PostContact>>().len() > 0 {
        return Err(Status::ExpectationFailed)
    };
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
    if let (Entitlement::Borrows, _) = ForUser::<Contact>::from(user)
            .has_jurisdiction(id, &**db)
            .to_status()? {
        return Err(Status::Unauthorized)
    }
    if contact.validate() == false {
        return Err(Status::ExpectationFailed)
    }
    contact.into_inner()
        .update(&**db, id)
        .to_status()?
        .to_json()
}

#[get("/contacts/public?<q>&<page>&<buffer>")]
pub fn search_public (db: State<DBState>, q: String, page: u32, buffer: Option<u32>) -> JsonResponse {
    let buffer = buffer.unwrap_or(10);
    Contact::search_public(&**db, page as i64, buffer as i64, q)
        .to_status()?
        .to_json()
}

#[get("/contacts/public/<id>")]
pub fn get_public_by_id (db: State<DBState>, id: i64) -> JsonResponse {
    let contact = Contact::force_get_by_id(id, &**db)
        .to_status()?;
    if contact.visibility() != Visibility::Public {
        return Err(Status::Unauthorized)
    }
    contact
        .get_full_info(&**db)
        .to_status()?
        .to_json()
}

#[derive(Serialize, Deserialize)]
struct FullResponse {
    contact: Contact,
    info: BareInfo
}

#[get("/contacts/private?<key>")]
pub fn get_by_key (db: State<DBState>,
                           persona_key: State<ContactJwtHandler>,
                           key: String,
                           user: UserId) -> JsonResponse {
    let contact = (*persona_key).extract (&key)
        .to_status()?
        .custom.id;

    let contact = Contact::force_get_by_id(contact, &db)
        .to_status()?;

    UserContactRelation (
        *user,
        contact.id
    ).register (&db)
        .to_status()?;

    FullResponse {
        info: contact.get_all_info(&**db)
            .to_status()?
            .info,
        contact
    }.to_json()
}

#[get("/contacts/key?<id>")]
pub fn get_key (db: State<DBState>, contact_key: State<ContactJwtHandler>, id: i64, user: UserId) -> JsonResponse {
    let contact = ForUser::<Contact>::from(user).query_by_id(id, &db)
        .to_status()?;

    if contact.creator != *user && contact.visibility() != Visibility::Public {
        return Err(Status::Unauthorized);
    }

    ContactJwt { id: contact.id }.encode(&contact_key.key)
        .to_status()?
        .to_json()
}
