use crate::{db::{DBState, Register, persona::NewPersona, user::{IsUser, NewUser, Password, User}}, derive_password, verification::jwt::{JwtHandler, LoginHandler, jwt_data::JwtData}};
use crate::rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};
use super::{Catch, SUCCESS, StatusCatch, ToStatus};
use crate::verification::{*, jwt::Token};

use super::EmptyResponse;
use crate::routing::{JsonResponse, ToJson};

#[derive(Serialize, Deserialize, Clone)]
pub struct RegisterUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

impl From<&RegisterUser> for NewUser {
    fn from(r: &RegisterUser) -> Self {
        let r = r.clone();
        NewUser::new(r.username, r.email, r.password)
    }
}

#[post("/register", format = "application/json", data = "<user>")]
pub fn register (user: Json<RegisterUser>, db: State<DBState>, jwt_key: State<LoginHandler>) -> JsonResponse {
    let user = NewUser::from(&*user);
    let login_data = Login {
        username: user.username.clone(),
        password: user.password.clone()
    };

    match User::query_by_username(&user.username, &db) {
        Ok(_) => return Err(Status::UnprocessableEntity),
        Err(e) => if e != diesel::result::Error::NotFound {
            return Err(e.to_status());
        }
    }

    let user = user
        .encrypt()
        .salt()
        .catch(Status::InternalServerError)?;
    
    user.register(&db)
        .and_then(|user| NewPersona::new_default (user.id).register (&db))
        .to_status ()?;

    login(Json(login_data), db, jwt_key)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String
}

derive_password! (Login);

#[post("/login", format = "application/json", data = "<user>")]
pub fn login (user: Json<Login>, db: State<DBState>, jwt_key: State<LoginHandler>) -> JsonResponse {

    println! ("\t=> Logging in {}", user.username);

    let dbuser = User::query_by_username(&user.username, &db)
        .to_status()?;

    let user = user.encrypt ();

    let authorized = dbuser.password_cmp(&user)
        .catch(Status::InternalServerError)?;

    if !authorized {
        return Err(Status::Unauthorized)
    }

    println! ("\t=> Password is correct");

    let mut out: String = "".to_string();
    jwt_key.authorize(&mut out, dbuser)
        .catch(Status::InternalServerError)?;

    out.to_json()
}

#[post("/logout")]
pub fn logout (jwt_key: State<LoginHandler>, token: Token) -> EmptyResponse {

    let auth = token.0;

    let jwt = jwt_key.extract(&auth)
        .catch(Status::Unauthorized)?;

    println!("\t=> Logging out {}", jwt.custom.username);

    (&*jwt_key).blacklist(JwtData::new_from_claims (jwt, auth));

    SUCCESS
}

#[delete("/", format = "application/json", data = "<login>")]
pub fn delete (login: Json<Login>, jwt_key: State<LoginHandler>, db: State<DBState>, token: Token) -> EmptyResponse {
    let jwt = super::Verifier::verify_or_respond(&*jwt_key, &token)?;

    let user = User::query_by_username(&login.username, &**db)
        .to_status()?;

    if user.id != jwt.custom.user_id && user.level < 1 {
        return Err(Status::Unauthorized)
    }

    let auth = login
        .encrypt ()
        .password_cmp (&user)
        .catch(Status::InternalServerError)?;

    if !auth {
        return Err(Status::Unauthorized)
    }

    user
        .delete(&db)
        .to_status()?;

    Ok(())
}