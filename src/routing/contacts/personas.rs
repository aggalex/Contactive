use rocket::{State, http::Status, response::Redirect};
use rocket_contrib::json::Json;
use crate::{db::{DBState, QueryById, Register, contact::{Contact, IsContact, NewContact, UserContactRelation}, persona::{NewPersona, Persona}, user::{IsUser, User}}, routing::{JsonResponse}, verification::jwt::{Jwt, JwtHandler, LoginHandler, persona_jwt::{PersonaJwt, PersonaJwtHandler}}};
use serde::{Deserialize, Serialize};
use crate::routing::{ToJson, Catch};
use super::super::Verifier;
use crate::routing::StatusCatch;
use crate::verification::jwt::Token;

#[get("/personas/<user>")]
pub fn get_personas_of_user (db: State<DBState>, jwt_key: State<LoginHandler>, user: i64, token: Token) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond (&token)?;

    let user = User::query_by_id(user, &**db)
        .to_status()?;

    if user.id == jwt.custom.user_id { 
        user.get_personas (&**db) 
    } else {
        user.get_public_personas (&**db)
    }
        .to_status()?
        .to_json()
}

#[get("/personas")]
pub fn get_personas (db: State<DBState>, jwt_key: State<LoginHandler>, token: Token) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond (&token)
        .catch(Status::Unauthorized)?;

    get_personas_of_user(db, jwt_key, jwt.custom.user_id, token)
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

#[post("/personas", format = "application/json", data = "<personas>")]
pub fn add_personas (db: State<DBState>, jwt_key: State<LoginHandler>, personas: Json<Vec<PostPersona>>, token: Token) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond (&token)?;

    (&*personas).into_iter().map(|persona|
        persona
            .to_new_persona (jwt.custom.user_id)
            .register(&db)
            .and_then(|persona| NewContact::new_default_from_persona (persona.id)
                .register(&db))
    )
    .collect::<Result<Vec<Contact>, _>> ()
    .to_status ()?
    .to_json()
}

#[get("/personas?<key>")]
pub fn get_persona_by_key (db: State<DBState>,
                           jwt_key: State<LoginHandler>,
                           persona_key: State<PersonaJwtHandler>,
                           key: String,
                           token: Token) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond (&token)?;

    let persona_id = (*persona_key).extract (&key)
        .to_status()?
        .custom.0;

    let contact = Contact::of_persona(persona_id, &db)
        .to_status()?;

    UserContactRelation (
        jwt.custom.user_id,
        contact.id
    ).register (&db)
    .to_status()?;

    contact
        .get_all_info (&db)
        .to_status()?
        .to_json()
}

#[get("/personas/key?<id>")]
pub fn get_key_for_persona (db: State<DBState>, jwt_key: State<LoginHandler>, persona_key: State<PersonaJwtHandler>, id: i64, token: Token) -> JsonResponse {
    let jwt = (*jwt_key).verify_or_respond(&token)?;

    let persona = Persona::query_by_id(id, &db)
        .to_status()?;
    
    if persona.user_id != jwt.custom.user_id && persona.private {
        return Err(Status::Unauthorized);
    }

    PersonaJwt(persona.id).encode(&persona_key.key)
        .to_status()?
        .to_json()
}