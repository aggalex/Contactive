use std::{marker::PhantomData, error::Error};

use jwt_simple::{prelude::{Claims, Duration, HS256Key, JWTClaims, MACLike}};
use rocket::http::{Cookie, Cookies, Status};
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

pub struct LoginHandler<'a> {
    pub key: HS256Key,
    pub blacklist: ThreadBlacklist,
    phantom: PhantomData<&'a ()>
}

impl<'a> LoginHandler<'a> {

    pub fn new () -> Self {
        Self {
            key: HS256Key::from_bytes("abcd".as_bytes()),
            blacklist: ThreadBlacklist::new (),
            phantom: PhantomData
        }
    }

}

impl<'a> JwtHandler<&String, LoginJwt> for LoginHandler<'a> {

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

impl<'a, 'c> JwtHandler<Cookie<'c>, LoginJwt> for LoginHandler<'a> {

    fn extract(&self, cookie: Cookie<'c>) -> Result<JWTClaims<LoginJwt>, jwt_simple::Error> {
        self.extract(&cookie.value ().to_string ())
    }

}

impl<'a> Blacklist for LoginHandler<'a> {

    type Data = JwtData;

    fn blacklist (&self, data: Self::Data) {
        self.blacklist.blacklist(data)
    }

    fn is_blacklisted (&self, token: &String) -> bool {
        self.blacklist.is_blacklisted(token)
    }
}

pub const AUTH_COOKIE_NAME: &str = "authentication";

impl<'a> Verifier for LoginHandler<'a> {

    type Data = user::User;

    type Ok = JWTClaims<LoginJwt>;

    type Err = jwt_simple::Error;

    type Source = Cookies<'a>;

    fn verify (&self, cookies: &mut Cookies) -> Result<JWTClaims<LoginJwt>, jwt_simple::Error> {
        match cookies.get_private(AUTH_COOKIE_NAME) {
            Some(cookie) => self.extract(cookie),
            None => Err(jwt_simple::Error::msg("Unauthorized"))
        }
    }

    fn authorize (&self, cookies: &mut Cookies, user: User) -> Result<(), Box<dyn Error>> {
        let key = LoginJwt::new_from_user (user).encode (&self.key)?;

        println! ("\t=> {}", (key));

        cookies.add_private(Cookie::build (AUTH_COOKIE_NAME, key.clone ())
            // .secure(true)        // WORKS ONLY WITH HTTPS
            .http_only(true)
            .expires({
                let mut exp_date = time::now();
                exp_date.tm_hour += 2;
                exp_date
            })
            .finish()
        );

        Ok(())
    }
}

impl ToStatus for jwt_simple::Error {
    fn to_status (&self) -> rocket::http::Status {
        Status::InternalServerError
    }
}