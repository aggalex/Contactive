use diesel::QueryDsl;
use rocket::{State, http::Cookies, response::Redirect};
use rocket_contrib::json::Json;
use crate::{db::{DBState, Register, contact::{Contact, NewContact}, persona::{NewPersona, Persona}, schema::{contacts, personas, users}, user::User}, jwt::JwtState};
use crate::diesel::*;
use serde::{Deserialize, Serialize};

use super::ContactsResponse;

#[derive(Queryable, Serialize, Deserialize)]
struct FullPersona {
    pub contact: Contact,
    pub persona: Persona
}

#[get("/<user>/personas")]
pub fn get_personas_of_user (db: State<DBState>, jwt_key: State<JwtState>, user: i64, cookies: Cookies) -> ContactsResponse {
    let jwt = if let Ok(user) = (*jwt_key).verify (cookies) { user } else {
        return ContactsResponse::Unauthorized(())
    };

    let user = if let Ok(user) = users::table
        .filter(users::id.eq(user))
        .limit(1)
        .load::<User> (&**db) 
    { user[0].clone () } else {
        return ContactsResponse::NotFound("Requested user was not found".to_string ())
    };

    let reply = if let Ok(personas) = contacts::table
        .inner_join(personas::table)
        .filter(personas::user_id.eq(user.id))
        .load::<FullPersona> (&**db) 
    {
        let personas = if user.id == jwt.custom.user_id { personas } else {
            personas.into_iter()
                .filter(|entry| entry.persona.private)
                .collect::<Vec<FullPersona>>()
        };
        
        if let Ok(reply) = serde_json::to_string(&personas) { reply } else {
            return ContactsResponse::InternalServerError("Serialization failed".to_string ())
        }
    } else {
        return ContactsResponse::InternalServerError("Query failed".to_string ())
    };

    ContactsResponse::Success(reply)
}

#[get("/personas")]
pub fn get_personas (jwt_key: State<JwtState>, cookies: Cookies) -> Redirect {
    let jwt = if let Ok(user) = (*jwt_key).verify (cookies) { user } else {
        return Redirect::temporary("/login")
    };

    Redirect::found(format!("/{}/personas", jwt.custom.user_id))
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
pub fn add_personas (db: State<DBState>, jwt_key: State<JwtState>, persona: Json<PostPersona>, cookies: Cookies) -> ContactsResponse {
    let jwt = if let Ok(user) = (*jwt_key).verify (cookies) { user } else {
        return ContactsResponse::Unauthorized(())
    };

    let contact = if let Ok (contact) = persona
        .to_new_persona (jwt.custom.user_id)
        .register(&**db)
        .and_then(|persona| NewContact::new_default_from_persona (persona.id)
            .register(&**db))
    { contact } else {
        return ContactsResponse::InternalServerError("Failed to create persona contact".to_string ())
    };

    ContactsResponse::Success(contact.id.to_string ())
}