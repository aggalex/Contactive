use rocket::http::Cookies;

use self::jwt::DefaultJwtHandler;

pub mod jwt;
pub trait Blacklist: Send + Sync {

    type Data;

    fn blacklist (&self, data: Self::Data);
    fn is_blacklisted (&self, token: &String) -> bool;

}
pub trait Verifier: Blacklist {

    type User;

    type Ok;
    type Err;

    fn verify (&self, cookies: Cookies) -> Result<Self::Ok, Self::Err>;
    fn authorize (&self, cookies: Cookies, user: Self::User) -> Result<Self::Ok, Self::Err>;

}

pub type DefaultVerificationHandler = DefaultJwtHandler;