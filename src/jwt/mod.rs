use blacklist::Blacklist;
use jwt_simple::prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike};
use rocket::http::{Cookie, Cookies};
use serde::{Serialize, Deserialize};

use crate::db::user::User;

use self::blacklist::JwtData;

pub mod blacklist;

#[derive(Clone, Serialize, Deserialize)]
pub struct Jwt {
    pub username: String,
    pub user_id: i64,
}

impl Jwt {

    pub fn new (username: String, user_id: i64) -> Jwt {
        Jwt {
            username,
            user_id
        }
    }

    pub fn new_from_user (user: User) -> Jwt {
        Jwt {
            username: user.username,
            user_id: user.id
        }
    }

    pub fn encode<Key: MACLike> (&self, key: &Key) -> Result<String, jwt_simple::Error> {
        key.authenticate(
            Claims::with_custom_claims (
                self.clone (),
                Duration::from_hours (2)
            )
        )
    }

}

impl PartialEq for Jwt {

    fn eq(&self, other: &Jwt) -> bool {
        self.username == other.username 
    }
}

pub struct JwtState {
    pub key: HS256Key,
    pub blacklist: Blacklist
}

impl JwtState {

    pub fn new () -> JwtState {
        JwtState {
            key: HS256Key::from_bytes("abcd".as_bytes()),
            blacklist: Blacklist::new ()
        }
    }

    pub fn extract (&self, token: &String) -> Result<JWTClaims<Jwt>, jwt_simple::Error> {
        let claims = match self.key.verify_token::<Jwt> (token, None) {
            Ok(claims) => claims,
            Err(err) => return Err(err)
        };
        
        if self.blacklist.contains (token) {
            return Err(jwt_simple::Error::msg ("Token is blacklisted"));
        };

        Ok(claims)    
    }

    pub fn cookie_extract (&self, cookie: Cookie) -> Result<JWTClaims<Jwt>, jwt_simple::Error> {
        self.extract(&cookie.value ().to_string ())
    }

    pub fn verify (&self, mut cookies: Cookies) -> Result<JWTClaims<Jwt>, jwt_simple::Error> {
        match cookies.get_private(crate::routing::user::AUTH_COOKIE_NAME) {
            Some(cookie) => self.cookie_extract(cookie),
            None => Err(jwt_simple::Error::msg("Unauthorized"))
        }
    }

    pub fn blacklist (&self, jwt: JwtData) {
        self.blacklist.blacklist(jwt);
        println!("{}", self.blacklist);
    }

}