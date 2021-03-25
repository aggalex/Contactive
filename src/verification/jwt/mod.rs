use jwt_simple::prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike};
use rocket::http::{Cookie, Cookies};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::db::user::{self, User};

use self::blacklist::ThreadBlacklist;
use self::jwt_data::JwtData;

use super::{Blacklist, Verifier};

pub mod jwt_data;
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

pub trait JwtHandler<J: Jwt>: Verifier<Data = JwtData, Ok = JWTClaims<J>, Err = jwt_simple::Error> {

    fn extract(&self, token: &String) ->  Result<Self::Ok, Self::Err>;
    fn cookie_extract (&self, cookie: Cookie) ->  Result<Self::Ok, Self::Err>;

}

impl JwtHandlerWithThreadBlacklist {

    pub fn new () -> Self {
        Self {
            key: HS256Key::from_bytes("abcd".as_bytes()),
            blacklist: ThreadBlacklist::new ()
        }
    }

}

impl JwtHandler<SimpleJwt> for JwtHandlerWithThreadBlacklist {

    fn extract (&self, token: &String) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        let claims = match self.key.verify_token::<SimpleJwt> (token, None) {
            Ok(claims) => claims,
            Err(err) => return Err(err)
        };
        
        if self.blacklist.is_blacklisted (token) {
            return Err(jwt_simple::Error::msg ("Token is blacklisted"));
        };

        Ok(claims)    
    }

    fn cookie_extract (&self, cookie: Cookie) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        self.extract(&cookie.value ().to_string ())
    }

}

impl Blacklist for JwtHandlerWithThreadBlacklist {

    type Data = JwtData;

    fn blacklist (&self, data: Self::Data) {
        self.blacklist.blacklist(data)
    }

    fn is_blacklisted (&self, token: &String) -> bool {
        self.blacklist.is_blacklisted(token)
    }
}

pub const AUTH_COOKIE_NAME: &str = "authentication";

impl Verifier for JwtHandlerWithThreadBlacklist {

    type User = user::User;

    type Ok = JWTClaims<SimpleJwt>;

    type Err = jwt_simple::Error;

    fn verify (&self, mut cookies: Cookies) -> Result<JWTClaims<SimpleJwt>, jwt_simple::Error> {
        match cookies.get_private(AUTH_COOKIE_NAME) {
            Some(cookie) => self.cookie_extract(cookie),
            None => Err(jwt_simple::Error::msg("Unauthorized"))
        }
    }

    fn authorize (&self, mut cookies: Cookies, user: User) -> Result<Self::Ok, Self::Err> {
        let key = DefaultJwt::new_from_user (user).encode (&self.key)?;

        println! ("\t=> {}", (key));

        cookies.add_private(Cookie::build (AUTH_COOKIE_NAME, key.clone ())
            // .secure(true)
            .http_only(true)
            .expires({
                let mut exp_date = time::now();
                exp_date.tm_hour += 2;
                exp_date
            })
            .finish()
        );

        self.verify(cookies)
    }
}

pub type DefaultJwtHandler = JwtHandlerWithThreadBlacklist;
pub type DefaultJwt = SimpleJwt;