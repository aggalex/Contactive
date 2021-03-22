use blacklist::Blacklist;
use jwt_simple::prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike};
use rocket::http::{Cookie, Cookies};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::db::user::User;

use self::blacklist::{JwtData, ThreadBlacklist};

use super::Verifier;

pub mod blacklist;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimpleJwt {
    pub username: String,
    pub user_id: i64,
}

impl SimpleJwt {

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

impl Jwt for SimpleJwt {

    fn encode<Key: MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error> {
        key.authenticate(
            Claims::with_custom_claims (
                self.clone (),
                Duration::from_hours (2)
            )
        )
    }

}

impl PartialEq for SimpleJwt {

    fn eq(&self, other: &SimpleJwt) -> bool {
        self.username == other.username 
    }
}

pub struct JwtHandlerWithThreadBlacklist {
    pub key: HS256Key,
    pub blacklist: ThreadBlacklist
}

pub trait JwtHandler<J: Jwt>: Verifier<Ok = JWTClaims<J>, Err = jwt_simple::Error> {

    fn extract(&self, token: &String) ->  Result<Self::Ok, Self::Err>;
    fn cookie_extract (&self, cookie: Cookie) ->  Result<Self::Ok, Self::Err>;

}

#[macro_export]
macro_rules! jwt_handler_default_ok_err {
    ($jwt: path) => {
        type Ok = JWTClaims<$jwt>;
        type Err = jwt_simple::Error;    
    };
}

impl JwtHandlerWithThreadBlacklist {

    pub fn new () -> Self {
        Self {
            key: HS256Key::from_bytes("abcd".as_bytes()),
            blacklist: ThreadBlacklist::new ()
        }
    }

    pub fn blacklist (&self, jwt: JwtData) {
        self.blacklist.blacklist(jwt);
        println!("{}", self.blacklist);
    }

}

impl JwtHandler<SimpleJwt> for JwtHandlerWithThreadBlacklist {

    fn extract (&self, token: &String) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        let claims = match self.key.verify_token::<SimpleJwt> (token, None) {
            Ok(claims) => claims,
            Err(err) => return Err(err)
        };
        
        if self.blacklist.contains (token) {
            return Err(jwt_simple::Error::msg ("Token is blacklisted"));
        };

        Ok(claims)    
    }

    fn cookie_extract (&self, cookie: Cookie) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        self.extract(&cookie.value ().to_string ())
    }

}

impl Verifier for JwtHandlerWithThreadBlacklist {

    jwt_handler_default_ok_err! (SimpleJwt);

    fn verify (&self, mut cookies: Cookies) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        match cookies.get_private(crate::routing::user::AUTH_COOKIE_NAME) {
            Some(cookie) => self.cookie_extract(cookie),
            None => Err(jwt_simple::Error::msg("Unauthorized"))
        }
    }

}

pub type DefaultJwtHandler = JwtHandlerWithThreadBlacklist;
pub type DefaultJwt = SimpleJwt;