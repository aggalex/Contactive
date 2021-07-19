use std::error::Error;

use jwt_simple::{prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike}};
use rocket::{http::Status, request::FromRequest};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{db::user::{self, User}, routing::ToStatus};

use self::blacklist::ThreadBlacklist;
use self::jwt_data::JwtData;

use super::{Blacklist, Verifier};

pub mod jwt_data;
pub mod blacklist;
pub mod persona_jwt;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginJwt {
    pub username: String,
    pub user_id: i64,
}

impl LoginJwt {

    pub fn new (username: String, user_id: i64) -> Self {
        Self {
            username,
            user_id
        }
    }

    pub fn new_from_user (user: User) -> Self {
        Self {
            username: user.username,
            user_id: user.id
        }
    }

}

pub trait Jwt: Clone + Serialize + DeserializeOwned + PartialEq {

    fn encode<Key: MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error>;

}

impl Jwt for LoginJwt {

    fn encode<Key: MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error> {
        key.authenticate(
            Claims::with_custom_claims (
                self.clone (),
                Duration::from_hours (2)
            )
        )
    }

}

impl PartialEq for LoginJwt {

    fn eq(&self, other: &LoginJwt) -> bool {
        self.username == other.username 
    }
}

pub trait JwtHandler<Source, J: Jwt>: Verifier<Ok = JWTClaims<J>, Err = jwt_simple::Error> {

    fn extract(&self, token: Source) ->  Result<Self::Ok, Self::Err>;

}

pub struct LoginHandler {
    pub key: HS256Key,
    pub blacklist: ThreadBlacklist,
}

impl LoginHandler {

    pub fn new () -> Self {
        Self {
            key: HS256Key::from_bytes("abcd".as_bytes()),
            blacklist: ThreadBlacklist::new (),
        }
    }

}

impl JwtHandler<&String, LoginJwt> for LoginHandler {

    fn extract (&self, token: &String) -> Result<JWTClaims<LoginJwt>, jwt_simple::Error> {
        let claims = match self.key.verify_token::<LoginJwt> (token, None) {
            Ok(claims) => claims,
            Err(err) => return Err(err)
        };
        
        if self.blacklist.is_blacklisted (token) {
            return Err(jwt_simple::Error::msg ("Token is blacklisted"));
        };

        Ok(claims)    
    }

}

impl<'a, 'c> JwtHandler<Token, LoginJwt> for LoginHandler {

    fn extract(&self, token: Token) -> Result<JWTClaims<LoginJwt>, jwt_simple::Error> {
        self.extract(&token.0)
    }

}

impl<'a> Blacklist for LoginHandler {

    type Data = JwtData;

    fn blacklist (&self, data: Self::Data) {
        self.blacklist.blacklist(data)
    }

    fn is_blacklisted (&self, token: &String) -> bool {
        self.blacklist.is_blacklisted(token)
    }
}

pub const AUTH_HEADER_NAME: &str = "authentication";

pub struct Token (pub String);

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = jwt_simple::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.headers().get(AUTH_HEADER_NAME).next() {
            Some(auth) => rocket::request::Outcome::Success(Token(auth.to_string())),
            None => rocket::request::Outcome::Failure((Status::Unauthorized, jwt_simple::Error::msg("Unauthorized"),))
        }
    }
}

impl Verifier for LoginHandler {

    type Data = user::User;

    type Ok = JWTClaims<LoginJwt>;

    type Err = jwt_simple::Error;

    type Source = Token;

    type Destination = String;

    fn verify (&self, token: &Token) -> Result<JWTClaims<LoginJwt>, jwt_simple::Error> {
        println!("\t=> Verifying for token: {}", token.0);
        self.extract(&token.0)
    }

    fn authorize (&self, key: &mut String, user: User) -> Result<(), Box<dyn Error>> {
        *key = LoginJwt::new_from_user (user).encode (&self.key)?;

        println! ("\t=> {}", key);
        
        Ok(())
    }
}

impl ToStatus for jwt_simple::Error {
    fn to_status (&self) -> rocket::http::Status {
        Status::InternalServerError
    }
}