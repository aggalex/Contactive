#![feature(proc_macro_hygiene, decl_macro, associated_type_defaults, generic_associated_types)]
#![recursion_limit="512"]

use dotenv::dotenv;

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
extern crate serde;
extern crate serde_json;
extern crate dotenv;
extern crate bcrypt;
extern crate sha2;
extern crate base64;
extern crate jwt_simple;
extern crate time;
extern crate sorted_vec;
extern crate chrono;
extern crate rocket_cors;

pub mod db;
pub mod routing;
pub mod verification;

mod tests;

fn main() {
    dotenv().ok();
    routing::start ().launch ();
}