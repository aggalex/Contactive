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

    fn reauthorize(&self, source: &Self::Source, destination: &mut Self::Destination) -> Result<(), Box<dyn Error>>;
    fn verify (&self, source: &Self::Source) -> Result<Self::Ok, Self::Err>;
    fn authorize<G> (&self, destination: &mut Self::Destination, data: G) -> Result<(), Box<dyn Error>>
        where <Self as Verifier>::Data: From<G>;

}