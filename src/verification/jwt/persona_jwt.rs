use jwt_simple::prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike};
use serde::{Deserialize, Serialize};
use crate::{db::persona::Persona, verification::{Blacklist, Verifier}};

use super::{Jwt, JwtHandler};
use crate::db::contact::{Contact, UserContactRelation};
use crate::db::{DBState, Register};
use crate::db::user::User;
use std::error::Error;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonaJwt (pub i64);

impl Jwt for PersonaJwt {
    fn encode<Key: jwt_simple::prelude::MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error> {
        key.authenticate(
            Claims::with_custom_claims (
                self.clone (),
                Duration::from_days (1)
            )
        )
    }
}

pub struct PersonaJwtHandler {
    pub key: HS256Key
}

impl JwtHandler<&String, PersonaJwt> for PersonaJwtHandler {
    fn extract(&self, token: &String) ->  Result<Self::Ok, Self::Err> {
        self.verify(&mut token.clone())
    }
}

impl Verifier for PersonaJwtHandler {
    type Data = Persona;
    type Ok = JWTClaims<PersonaJwt>;
    type Err = jwt_simple::Error;
    type Source = String;
    type Destination = (User, DBState);

    fn verify (&self, token: &mut String) -> Result<Self::Ok, Self::Err> {
        self.key.verify_token::<PersonaJwt> (token, None)
    }

    fn authorize (&self, (user, db): &mut (User, DBState), item: Persona) -> Result<(), Box<dyn Error>> {
        let contact = Contact::of_persona (item.id, db)?;

        UserContactRelation (
            user.id,
            contact.id
        ).register (db)?;

        Ok(())
    }
}

// No blacklist
impl Blacklist for PersonaJwtHandler {
    type Data = PersonaJwt;

    fn blacklist (&self, _: Self::Data) {
        
    }

    fn is_blacklisted (&self, _: &String) -> bool {
        false
    }
}