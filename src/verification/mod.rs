use rocket::http::Cookies;

pub mod jwt;

pub trait Verifier {

    type Ok;
    type Err;

    fn verify (&self, cookies: Cookies) -> Result<Self::Ok, Self::Err>;

}
