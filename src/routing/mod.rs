use rocket::{Rocket, http::Status, Request, request::Outcome};
use serde::Serialize;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};

use crate::verification::jwt::{LoginHandler, Token};
use rocket::request::FromRequest;
use crate::db::user::UserId;

pub mod user;
pub mod contacts;

#[get("/")]
fn root() -> String {
    format!("Welcome to Rocket on Rust")
}

pub fn start () -> Rocket {
    rocket::ignite()
    .manage(crate::db::DBState::new ())
    .manage(LoginHandler::new ())
    .mount("/", routes![
        root,
        user::register,
        user::login,
        user::logout,
        user::delete,
        user::me,
        user::renew,
        contacts::get_contacts,
        contacts::add_contacts,
        contacts::delete_contact,
        contacts::edit_contact,
        contacts::search_public,
        contacts::get_by_key,
        contacts::get_key,
        contacts::info::get_info,
        contacts::info::post_info_by_data,
        contacts::info::post_info_by_url,
        contacts::info::delete_info,
        contacts::info::patch_info,
    ]).attach(CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Delete, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors().unwrap()
    )
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct JsonResponseOk (String);

type JsonResponse = Result<JsonResponseOk, Status>;

trait JsonResponseNew {
    fn new<T: ?Sized + Serialize> (data: &T) -> JsonResponse;
}

impl JsonResponseNew for JsonResponse {

    fn new<T: ?Sized + Serialize> (data: &T) -> JsonResponse {
        Ok(JsonResponseOk(serde_json::to_string(data)
            .catch(Status::InternalServerError)?))
    }

}

trait ToJson: Serialize {

    fn to_json (&self) -> JsonResponse {
        JsonResponse::new(self)
    }

}

impl<T: ?Sized + Serialize> ToJson for T {}

type EmptyResponse = Result<(), Status>;

const SUCCESS: EmptyResponse = Ok(());

trait Verifier: crate::verification::Verifier {

    fn verify_or_respond (&self, cookies: &Self::Source) -> Result<Self::Ok, Status> {
        self.verify (cookies).catch(Status::Unauthorized)
    }

}

impl<'a, 'r> FromRequest<'a, 'r> for UserId {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, ()> {
        let key = request.guard::<rocket::State::<LoginHandler>>()?;
        let token = request.guard::<Token>().map_failure(|_| (Status::Unauthorized, ()))?;

        match key.verify_or_respond (&token) {
            Ok(claims) => Outcome::Success(UserId::new(claims.custom.user_id)),
            Err(_) => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

impl<V: crate::verification::Verifier> Verifier for V {}

pub trait ToStatus {
    fn to_status (&self) -> Status;
}

trait StatusCatch<Ok, Err: ToStatus + std::fmt::Debug>: Catch<Ok> {
    fn to_status (self) -> Result<Ok, Status>;
}

trait Catch<Ok> {
    fn catch<NE> (self, err: NE) -> Result<Ok, NE>;
}

impl<Ok, Err: std::fmt::Debug> Catch<Ok> for Result<Ok, Err> {
    fn catch<NE> (self, err: NE) -> Result<Ok, NE> {
        self.map_err(|e| {
            println!("\t=>\u{001b}[1;31m {:?}\u{001b}[0m", e);
            err
        })
    }
}

impl<Ok, Err: std::fmt::Debug + ToStatus> StatusCatch<Ok, Err> for Result<Ok, Err> {
    fn to_status (self) -> Result<Ok, Status> {
        self.map_err(|err| {
            println!("\t=>\u{001b}[1;31m {:?}\u{001b}[0m", err);
            err.to_status ()
        })
    }
}