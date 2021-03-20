#![feature(proc_macro_hygiene, decl_macro)]
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

pub mod db;
pub mod routing;
pub mod jwt;

fn main() {
    dotenv().ok();
    routing::start ()
}