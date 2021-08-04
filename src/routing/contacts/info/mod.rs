use rocket::{State, http::Status};
use rocket_contrib::json::Json;
use serde::{ Serialize, Deserialize };

use crate::{db::{ConjuctionTable, DBState, QueryById, contact::{Contact, IsContact, UserContactRelation, info::{BareInfo, Info}}}, routing::{Catch, EmptyResponse, JsonResponse, SUCCESS, ToJson, Verifier}, verification::jwt::LoginHandler};
use crate::verification::jwt::Token;
use crate::routing::StatusCatch;
use crate::db::contact::info::{InfoFragment, InfoSection};
use std::collections::HashMap;
use crate::db::{DefaultBackend, DefaultConnection};

#[get("/info/<contact>")]
pub fn get_info (db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64,
                 token: Token) -> JsonResponse {
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

fn _check_post_auth (db: &DefaultConnection, user: i64, contact_id: i64) -> Result<(), Status> {
    let contact = Contact::query_by_id(contact_id, db)
        .catch(Status::InternalServerError)?;

    let dbuser = contact.get_user (db)
        .catch(Status::InternalServerError)?;

    if dbuser.id != user {
        return Err(Status::Unauthorized)
    };

    Ok(())
}

fn _post_info (db: State<DBState>, jwt_key: State<LoginHandler>, info: Info, token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    _check_post_auth(&**db, user.custom.user_id, info.contact_id)?;

    info.register(&db)
        .catch(Status::InternalServerError)?;

    SUCCESS
}

#[post("/info", format = "application/json", data = "<info>")]
pub fn post_info_by_data (db: State<DBState>, jwt_key: State<LoginHandler>, info: Json<Info>,
                          token: Token) -> EmptyResponse {
    _post_info (db, jwt_key, info.clone(), token)
}

#[post("/info/<contact>", format = "application/json", data = "<info>")]
pub fn post_info_by_url (db: State<DBState>, jwt_key: State<LoginHandler>,
                         contact: i64, info: Json<BareInfo>, token: Token) -> EmptyResponse {
    _post_info (db, jwt_key, Info {
        contact_id: contact,
        info: info.clone ()
    }, token)
}

#[delete("/info/<contact>", format = "application/json", data = "<infosections>")]
pub fn delete_info(db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64,
                   infosections: Json<HashMap<String, Option<Vec<String>>>>,
                   token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    _check_post_auth(&**db, user.custom.user_id, contact)?;

    _delete_info(&**db, &*infosections, contact)?;

    SUCCESS
}

fn _delete_info(db: &DefaultConnection,
                infosections: &HashMap<String, Option<Vec<String>>>,
                contact: i64) -> EmptyResponse {
    (&*infosections).into_iter()
        .filter_map(|(k, values)| if let Some(v) = values {
            Some(Ok(v.into_iter()
                .map(|v| (k.clone(), v.clone()))
                .collect::<Vec<(String, String)>>()))
        } else {
            InfoSection {
                name: k.clone(),
                contact
            }.delete(db).map_err(|e| Err(e)).err()
        })
        .collect::<Result<Vec<Vec<(String, String)>>, diesel::result::Error>>()
        .to_status()?
        .into_iter()
        .flatten()
        .map(|(key, value)|
            InfoFragment {
                key,
                value,
                contact_id: contact
            }.delete(db))
        .collect::<Result<Vec<()>, diesel::result::Error>>()
        .to_status()?;
    SUCCESS
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Diff {
    delete: HashMap<String, Option<Vec<String>>>,
    new: BareInfo,
}

#[patch("/info/<contact>", format = "application/json", data = "<infosections>")]
pub fn patch_info(db: State<DBState>, jwt_key: State<LoginHandler>, contact: i64,
                   infosections: Json<Diff>,
                   token: Token) -> EmptyResponse {
    let user = (*jwt_key).verify_or_respond (&token)?;

    let infosections = infosections.into_inner();

    let info = Info {
        contact_id: contact,
        info: infosections.new
    };

    _check_post_auth(&**db, user.custom.user_id, contact)?;

    _delete_info(&**db, &infosections.delete, contact)?;

    info.register(&**db)
        .to_status()?;

    SUCCESS
}