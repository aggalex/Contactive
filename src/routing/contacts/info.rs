use rocket::{State, http::Status};
use rocket_contrib::json::Json;
use serde::{ Serialize, Deserialize };

use crate::{db::{DBState, contact::{Contact, IsContact, info::{BareInfo, Info}}}, routing::{Catch, EmptyResponse, JsonResponse, SUCCESS, ToJson}};
use crate::routing::StatusCatch;
use crate::db::contact::info::{InfoFragment, InfoSection, Jurisdiction};
use std::collections::HashMap;
use crate::db::{DefaultConnection, Delete};
use crate::db::user::{UserId, ForUser};

#[get("/info/<contact>")]
pub fn get_info (db: State<DBState>, contact: i64,
                 user: UserId) -> JsonResponse {
    let factory = ForUser::<Contact>::from(user);

    let contact = factory.query_by_id(contact, &**db)
        .catch(Status::InternalServerError)?;

    contact
        .get_all_info (&**db)
        .to_status()?
        .to_json ()
}

fn _check_post_auth (db: &DefaultConnection, user: UserId, contact_id: i64) -> Result<(), Status> {
    ForUser::<Contact>::from(user).query_by_id(contact_id, db)
        .to_status()?;

    Ok(())
}

fn _post_info (db: State<DBState>, info: Info, user: UserId) -> EmptyResponse {

    _check_post_auth(&**db, user, info.contact_id)?;

    info.register(&db)
        .catch(Status::InternalServerError)?;

    SUCCESS
}

#[post("/info", format = "application/json", data = "<info>")]
pub fn post_info_by_data (db: State<DBState>, info: Json<Info>, user: UserId) -> EmptyResponse {
    _post_info (db, info.into_inner(), user)
}

#[post("/info/<contact>", format = "application/json", data = "<info>")]
pub fn post_info_by_url (db: State<DBState>, contact: i64, info: Json<BareInfo>, user: UserId) -> EmptyResponse {
    _post_info (db, Info {
        contact_id: contact,
        info: info.clone ()
    }, user)
}

#[delete("/info/<contact>", format = "application/json", data = "<infosections>")]
pub fn delete_info(db: State<DBState>, contact: i64,
                   infosections: Json<HashMap<String, Option<Vec<String>>>>,
                   user: UserId) -> EmptyResponse {
    _check_post_auth(&**db, user, contact)?;

    _delete_info(&**db, &*infosections, contact, user)?;

    SUCCESS
}

fn _delete_info(db: &DefaultConnection,
                infosections: &HashMap<String, Option<Vec<String>>>,
                contact: i64,
                user: UserId) -> EmptyResponse {
    let mut sections = vec![];
    let fragments = (&*infosections).into_iter()
        .filter_map(|(k, values)| if let Some(v) = values {
            Some(Ok(v.into_iter()
                .map(|v| (k.clone(), v.clone()))
                .collect::<Vec<(String, String)>>()))
        } else {
            sections.push(InfoSection {
                name: k.clone(),
                contact
            });
            None
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
            })
        .collect::<Vec<InfoFragment>>();

    Jurisdiction::new(user, fragments, db)
        .catch(Status::Unauthorized)?
        .delete(db, ())
        .to_status()?;

    Jurisdiction::new(user, sections, db)
        .catch(Status::Unauthorized)?
        .delete(db, ())
        .to_status()?;

    SUCCESS
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Diff {
    delete: HashMap<String, Option<Vec<String>>>,
    new: BareInfo,
}

#[patch("/info/<contact>", format = "application/json", data = "<infosections>")]
pub fn patch_info(db: State<DBState>, contact: i64,
                   infosections: Json<Diff>,
                   user: UserId) -> EmptyResponse {

    let infosections = infosections.into_inner();

    let info = Info {
        contact_id: contact,
        info: infosections.new
    };

    _check_post_auth(&**db, user, contact)?;

    _delete_info(&**db, &infosections.delete, contact, user)?;

    info.register(&**db)
        .to_status()?;

    SUCCESS
}