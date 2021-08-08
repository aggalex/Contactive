use jwt_simple::prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike};
use serde::{Deserialize, Serialize};
use crate::verification::{Blacklist, Verifier};

use super::{Jwt, JwtHandler};
use crate::db::contact::{Contact, UserContactRelation};
use crate::db::{DBState, Register};
use crate::db::user::User;
use std::error::Error;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactJwt {
    pub id: i64,
}

impl Jwt for ContactJwt {
    fn encode<Key: jwt_simple::prelude::MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error> {
        key.authenticate(
            Claims::with_custom_claims (
                self.clone (),
                Duration::from_days (1)
            )
        )
    }
}

pub struct ContactJwtHandler {
    pub key: HS256Key
}

impl ContactJwtHandler {
    pub fn new() -> ContactJwtHandler {
        ContactJwtHandler {
            key: HS256Key::from_bytes("abcd".as_bytes())
        }
    }
}

impl JwtHandler<&String, ContactJwt> for ContactJwtHandler {
    fn extract(&self, token: &String) ->  Result<Self::Ok, Self::Err> {
        self.verify(&mut token.clone())
    }
}

impl Verifier for ContactJwtHandler {
    type Data = Contact;
    type Ok = JWTClaims<ContactJwt>;
    type Err = jwt_simple::Error;
    type Source = String;
    type Destination = (User, DBState);

    fn reauthorize(&self, source: &String, destination: &mut (User, DBState)) -> Result<(), Box<dyn Error>> {
        let (_, db) = destination;
        let claims = self.verify(source)?;
        let contact = Contact::force_get_by_id(claims.custom.id, db)?;
        self.authorize(destination, contact)
    }

    fn verify (&self, token: &String) -> Result<Self::Ok, Self::Err> {
        self.key.verify_token::<ContactJwt> (token, None)
    }

    fn authorize<G> (&self, (user, db): &mut (User, DBState), item: G) -> Result<(), Box<dyn Error>>
        where Contact: From<G>
    {
        let contact = Contact::from(item);

        UserContactRelation (
            user.id,
            contact.id
        ).register (db)?;

        Ok(())
    }
}

// No blacklist
impl Blacklist for ContactJwtHandler {
    type Data = ContactJwt;

    fn blacklist (&self, _: Self::Data) {
        
    }

    fn is_blacklisted (&self, _: &String) -> bool {
        false
    }
}