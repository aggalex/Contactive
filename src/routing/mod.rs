use rocket::{Rocket, http::{Cookies, Status}};
use serde::Serialize;

use crate::verification::DefaultVerificationHandler;

pub mod user;
pub mod contacts;

#[get("/")]
fn root() -> String {
    format!("Welcome to Rocket on Rust")
}

pub fn start () -> Rocket {
    rocket::ignite()
    .manage(crate::db::DBState::new ())
    .manage(DefaultVerificationHandler::new ())
    .mount("/", routes![
        root,
        user::register,
        user::login,
        user::logout,
        user::delete,
        contacts::get_contacts,
        contacts::personas::get_personas,
        contacts::personas::get_personas_of_user
    ])
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
            .map_err(|_| Status::InternalServerError)?))
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

    fn verify_or_respond (&self, cookies: Cookies) -> Result<Self::Ok, Status> {
        self.verify (cookies).map_err(|_| Status::Unauthorized)
    }

}

impl<V: crate::verification::Verifier> Verifier for V {}