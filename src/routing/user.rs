use crate::{db::{DBState, Register, persona::NewPersona, schema::users, user::{NewUser, Password, User}}, derive_password};
use crate::rocket::State;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::{http::{Cookie, Cookies, Status}};
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};
use super::SUCCESS;
use time;
use crate::verification::{jwt::*, jwt::blacklist::JwtData};

use super::EmptyResponse;

pub const AUTH_COOKIE_NAME: &str = "authentication";

#[post("/register", format = "application/json", data = "<user>")]
pub fn register (user: Json<NewUser>, db: State<DBState>) -> EmptyResponse {

    let users = FilterDsl::filter(users::table, users::username.eq(&user.username))
        .limit(1)
        .load::<User>(&**db)
        .map_err(|_| Status::UnprocessableEntity)?;

    if users.len () > 0 {
        return Err(Status::UnprocessableEntity);
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
pub fn login (user: Json<Login>, db: State<DBState>, jwt_key: State<DefaultJwtHandler>, mut cookies: Cookies) -> EmptyResponse {

    println! ("\t=> Logging in {}", user.username);

    let users = FilterDsl::filter(users::table, users::username.eq(&user.username))
        .limit(1)
        .load::<User>(&**db)
        .map_err(|_| Status::InternalServerError)?;

    let dbuser = if users.len () < 1 {
        return Err(Status::NotFound)
    } else {
        users[0].clone ()
    };

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

    let key = DefaultJwt::new_from_user (dbuser).encode (&jwt_key.key)
        .map_err(|_| Status::InternalServerError)?;

    println! ("\t=> {}", (key));

    cookies.add_private(Cookie::build (AUTH_COOKIE_NAME, key.clone ())
        // .secure(true)
        .http_only(true)
        .expires({
            let mut exp_date = time::now();
            exp_date.tm_hour += 2;
            exp_date
        })
        .finish()
    );

    SUCCESS
}

#[post("/logout")]
pub fn logout (jwt_key: State<DefaultJwtHandler>, mut cookies: Cookies) -> EmptyResponse {

    let cookie = cookies.get_private(AUTH_COOKIE_NAME)
        .ok_or(Status::UnprocessableEntity)?;

    let auth = cookie.value ().to_string ();

    let jwt = jwt_key.extract(&auth)
        .map_err(|_| Status::Unauthorized)?;

    cookies.remove_private(Cookie::named (AUTH_COOKIE_NAME));

    (&*jwt_key).blacklist(JwtData::new_from_claims (jwt, auth));

    SUCCESS
}