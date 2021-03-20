use crate::{db::{DBState, Register, persona::NewPersona, schema::users, user::{NewUser, Password, User}}, derive_password, jwt::blacklist::JwtData};
use crate::rocket::State;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::{http::{Cookie, Cookies, Status}};
use rocket_contrib::json::Json;
use serde_json::json;
use serde::{Serialize, Deserialize};
use time;
use crate::jwt::*;

pub const AUTH_COOKIE_NAME: &str = "authentication";

#[post("/register", format = "application/json", data = "<user>")]
pub fn register (user: Json<NewUser>, db: State<DBState>) -> Status {

    if let Ok(users) = FilterDsl::filter(users::table, users::username.eq(&user.username))
        .limit(1)
        .load::<User>(&**db)
    {
        if users.len () > 0 {
            return Status::UnprocessableEntity;
        }
    } else {
        return Status::InternalServerError;
    };

    let user = if let Ok(u) = user.encrypt().salt() { u } else {
        return Status::InternalServerError
    };
    
    match user.register(&**db)
                .and_then(|user| NewPersona::new_default (user.id).register (&**db)) {
        Ok(_) => Status::Ok,
        Err(e) => {
            println!("\t=> Error: {}", e);
            Status::InternalServerError
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String
}

derive_password! (Login);

#[derive(Responder)]
pub enum LoginResponse {
    #[response(status = 500, content_type = "plain")]
    InternalServerError(String),
    #[response(status = 401, content_type = "plain")]
    Unauthorized(String),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 200)]
    Success(())
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login (user: Json<Login>, db: State<DBState>, jwt_key: State<JwtState>, mut cookies: Cookies) -> LoginResponse {

    println! ("\t=> Logging in {}", user.username);

    let dbuser: User = if let Ok(users) = FilterDsl::filter(users::table, users::username.eq(&user.username))
        .limit(1)
        .load::<User>(&**db)
    {
        if users.len () < 1 {
            return LoginResponse::NotFound(json!({
                "username": user.username
            }).to_string())
        } else {
            users[0].clone ()
        }
    } else {
        return LoginResponse::InternalServerError("Connection to database failed".to_string ())
    };

    println! ("\t=> Found in db: {}", dbuser.id);

    let user = user.encrypt ();

    println! ("\t=> encrypted password: {}", user.password);
    println! ("\t=> database  password: {}", dbuser.password);

    let authorized = if let Ok(cmp) = dbuser.password_cmp(&user) { cmp } else {
        return LoginResponse::InternalServerError("Failed to compare passwords".to_string ())
    };

    if !authorized {
        return LoginResponse::Unauthorized ("password is incorrect".to_string())
    }

    println! ("\t=> Password is correct");

    let key = if let Ok(key) = Jwt::new_from_user (dbuser).encode (&jwt_key.key) { key } else {
        return LoginResponse::InternalServerError("Failed to create jwt key".to_string ())
    };

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

    LoginResponse::Success(())
}

#[derive(Responder)]
pub enum LogoutResponse {
    #[response(status = 500, content_type = "plain")]
    InternalServerError(String),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 422)]
    UnprocessableEntity(String),
    #[response(status = 200)]
    Success(())
}

#[post("/logout")]
pub fn logout (jwt_key: State<JwtState>, mut cookies: Cookies) -> LogoutResponse {

    let cookie = if let Some(cookie) = cookies.get_private(AUTH_COOKIE_NAME) { cookie } else {
        return LogoutResponse::UnprocessableEntity("No authentication token found".to_string ())
    };

    let auth = cookie.value ().to_string ();

    let jwt = if let Ok(key) = jwt_key.extract(&auth) { key } else {
        return LogoutResponse::Unauthorized(())
    };

    cookies.remove_private(Cookie::named (AUTH_COOKIE_NAME));

    jwt_key.blacklist(JwtData::new_from_claims (jwt, auth));

    LogoutResponse::Success(())
}