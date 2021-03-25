use crate::{db::{DBState, Register, persona::NewPersona, user::{IsUser, NewUser, Password, User, UserDescriptor}}, derive_password, verification::jwt::{AUTH_COOKIE_NAME, JwtHandler, jwt_data::JwtData}};
use crate::rocket::State;
use rocket::{http::{Cookie, Cookies, Status}};
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};
use super::{SUCCESS};
use crate::verification::*;

use super::EmptyResponse;

#[post("/register", format = "application/json", data = "<user>")]
pub fn register (user: Json<NewUser>, db: State<DBState>) -> EmptyResponse {

    match User::query_by_username(&user.username, &**db) {
        Ok(_) => return Err(Status::UnprocessableEntity),
        Err(e) => if e != diesel::result::Error::NotFound {
            return Err(Status::InternalServerError);
        }
    }

    let user = user
        .encrypt()
        .salt()
        .map_err(|_| Status::InternalServerError)?;
    
    user.register(&**db)
        .and_then(|user| NewPersona::new_default (user.id).register (&**db))
        .map_err (|e| {
            println!("\t=> Error: {}", e);
            Status::InternalServerError
        })?;

    SUCCESS
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String
}

derive_password! (Login);

#[post("/login", format = "application/json", data = "<user>")]
pub fn login (user: Json<Login>, db: State<DBState>, jwt_key: State<DefaultVerificationHandler>, cookies: Cookies) -> EmptyResponse {

    println! ("\t=> Logging in {}", user.username);

    let dbuser = User::query_by_username(&user.username, &**db)
        .map_err(|_| Status::NotFound)?;

    println! ("\t=> Found in db: {}", dbuser.id);

    let user = user.encrypt ();

    println! ("\t=> encrypted password: {}", user.password);
    println! ("\t=> database  password: {}", dbuser.password);

    let authorized = dbuser.password_cmp(&user)
        .map_err(|_| Status::InternalServerError)?;

    if !authorized {
        return Err(Status::Unauthorized)
    }

    println! ("\t=> Password is correct");

    jwt_key.authorize(cookies, dbuser)
        .map_err(|_| Status::InternalServerError)?;

    SUCCESS
}

#[post("/logout")]
pub fn logout (jwt_key: State<DefaultVerificationHandler>, mut cookies: Cookies) -> EmptyResponse {

    let cookie = cookies.get_private(AUTH_COOKIE_NAME)
        .ok_or(Status::UnprocessableEntity)?;

    let auth = cookie.value ().to_string ();

    let jwt = jwt_key.extract(&auth)
        .map_err(|_| Status::Unauthorized)?;

    cookies.remove_private(Cookie::named (AUTH_COOKIE_NAME));

    (&*jwt_key).blacklist(JwtData::new_from_claims (jwt, auth));

    SUCCESS
}

#[delete("/")]
pub fn delete (jwt_key: State<DefaultVerificationHandler>, db: State<DBState>, cookies: Cookies) -> EmptyResponse {
    let jwt = super::Verifier::verify_or_respond(&*jwt_key, cookies)?;

    UserDescriptor(jwt.custom.user_id)
        .delete(&**db)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Status::NotFound,
            _ => Status::InternalServerError
        })?;

    Ok(())
}