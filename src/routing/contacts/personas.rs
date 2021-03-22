use diesel::QueryDsl;
use rocket::{State, http::{Cookies, Status}, response::Redirect};
use rocket_contrib::json::Json;
use crate::{db::{DBState, Register, contact::{Contact, NewContact}, persona::{NewPersona, Persona}, schema::{contacts, personas, users}, user::User}, verification::jwt::{DefaultJwtHandler}, routing::{EmptyResponse, JsonResponse, SUCCESS}};
use crate::diesel::*;
use serde::{Deserialize, Serialize};
use crate::routing::JsonResponseNew;

use super::super::Verifier;

#[derive(Queryable, Serialize, Deserialize)]
struct FullPersona {
    pub contact: Contact,
    pub persona: Persona
}

#[get("/personas/<user>")]
pub fn get_personas_of_user (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, user: i64, cookies: Cookies) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond (cookies)?;

    let user = users::table
        .filter(users::id.eq(user))
        .limit(1)
        .load::<User> (&**db)
        .map_err(|_| Status::NotFound)?[0]
        .clone ();

    let personas = contacts::table
        .inner_join(personas::table)
        .filter(personas::user_id.eq(user.id))
        .load::<FullPersona> (&**db)
        .map_err (|_| Status::NotFound)?;

    let personas = if user.id == jwt.custom.user_id { personas } else {
        personas.into_iter()
            .filter(|entry| entry.persona.private)
            .collect::<Vec<FullPersona>>()
    };

    JsonResponse::new (&personas)
}

#[get("/personas")]
pub fn get_personas (jwt_key: State<DefaultJwtHandler>, cookies: Cookies) -> Result<Redirect, Redirect> {
    let jwt = (*jwt_key).verify_or_respond (cookies)
        .map_err(|_| Redirect::temporary("/login"))?;

    Ok(Redirect::found(format!("/{}/personas", jwt.custom.user_id)))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostPersona {
    pub name: String,
    pub private: bool
}

impl PostPersona {
    
    pub fn to_new_persona (&self, user: i64) -> NewPersona {
        NewPersona::new(self.name.clone (), self.private, user)
    }

}

#[post("/personas", format = "application/json", data = "<persona>")]
pub fn add_personas (db: State<DBState>, jwt_key: State<DefaultJwtHandler>, persona: Json<PostPersona>, cookies: Cookies) -> EmptyResponse {
    let jwt = (*jwt_key).verify_or_respond (cookies)?;

    persona
        .to_new_persona (jwt.custom.user_id)
        .register(&**db)
        .and_then(|persona| NewContact::new_default_from_persona (persona.id)
            .register(&**db))
        .map_err(|_| Status::InternalServerError)?;

    SUCCESS
}