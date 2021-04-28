use crate::routing::ToStatus;
use std::error::Error;

pub mod jwt;
pub trait Blacklist: Send + Sync {

    type Data;

    fn blacklist (&self, data: Self::Data);
    fn is_blacklisted (&self, token: &String) -> bool;

}
pub trait Verifier: Blacklist {

    type Data;

    type Ok;
    type Err: ToStatus + std::fmt::Debug;

    type Source;
    type Destination = Self::Source;

    fn verify (&self, source: &mut Self::Source) -> Result<Self::Ok, Self::Err>;
    fn authorize (&self, destination: &mut Self::Destination, data: <Self as Verifier>::Data) -> Result<(), Box<dyn Error>>;

}