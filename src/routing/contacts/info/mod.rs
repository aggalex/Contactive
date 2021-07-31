use rocket::{State, http::Status};
use rocket_contrib::json::Json;

use crate::{db::{ConjuctionTable, DBState, QueryById, contact::{Contact, IsContact, UserContactRelation, info::{BareInfo, Info}}}, routing::{Catch, EmptyResponse, JsonResponse, SUCCESS, ToJson, Verifier}, verification::jwt::LoginHandler};
use crate::verification::jwt::Token;
use crate::routing::StatusCatch;
use crate::db::contact::info::InfoFragment;
use std::collections::HashMap;

#[get("/info/<contact>")]
pub fn get_info (db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64, token: Token) -> JsonResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    let contact = Contact::query_by_id(contact, &**db)
        .catch(Status::InternalServerError)?;

    UserContactRelation(user.custom.user_id, contact.id)
        .get_both(&**db)
        .catch(Status::Unauthorized)?;

    contact
        .get_all_info (&**db)
        .to_status()?
        .to_json ()
}

fn _post_info (db: State<DBState>, jwt_key: State<LoginHandler>, info: Info, token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    let contact = info.get_contact (&db)
        .catch(Status::InternalServerError)?;
    
    let dbuser = contact.get_user (&db)
        .catch(Status::InternalServerError)?;

    if dbuser.id != user.custom.user_id {
        return Err(Status::Unauthorized)
    }

    info.register(&db)
        .catch(Status::InternalServerError)?;

    SUCCESS
}

#[post("/info", format = "application/json", data = "<info>")]
pub fn post_info_by_data (db: State<DBState>, jwt_key: State<LoginHandler>, info: Json<Info>, token: Token) -> EmptyResponse {
    _post_info (db, jwt_key, info.clone(), token)
}

#[post("/info/<contact>", format = "application/json", data = "<info>")]
pub fn post_info_by_url (db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64, info: Json<BareInfo>, token: Token) -> EmptyResponse {
    _post_info (db, jwt_key, Info {
        contact_id: contact,
        info: info.clone ()
    }, token)
}

//TODO: optimize
#[delete("/info/<contact>", format = "application/json", data = "<infosections>")]
pub fn delete_info(db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64, infosections: Json<HashMap<String, Option<String>>>, token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    let contact = Contact::query_by_id(contact, &**db)
        .catch(Status::InternalServerError)?;

    UserContactRelation(user.custom.user_id, contact.id)
        .get_both(&**db)
        .catch(Status::Unauthorized)?;

    let info = contact
        .get_all_info (&**db)
        .to_status()?;

    info.info.into_iter()
        .filter(|(name, _)| infosections.get(name).is_some())
        .map(|(key, values)| values.into_iter()
            .filter(|v| if let Some(val) = &infosections[&key] { val == v } else { true })
            .map(|v| InfoFragment {
                key: key.clone(),
                value: v.clone(),
                contact_id: contact.id
            }.delete(&**db).to_status())
            .collect::<Result<_, Status>>())
        .collect::<Result<_, Status>>()
}