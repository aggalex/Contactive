use rocket::{State, http::Status};
use rocket_contrib::json::Json;

use crate::{db::{ConjuctionTable, DBState, QueryById, contact::{Contact, IsContact, UserContactRelation, info::{BareInfo, Info}}}, routing::{Catch, EmptyResponse, JsonResponse, SUCCESS, ToJson, Verifier}, verification::jwt::LoginHandler};
use crate::verification::jwt::Token;

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
        .catch(Status::InternalServerError)?
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