use crate::jwt::JwtState;

pub mod user;
pub mod contacts;

#[get("/")]
fn root() -> String {
    format!("Welcome to Rocket on Rust")
}

pub fn start () {
    rocket::ignite()
    .manage(crate::db::DBState::new ())
    .manage(JwtState::new ())
    .mount("/", routes![
        root,
        user::register,
        user::login,
        user::logout,
        contacts::get_contacts,
        contacts::personas::get_personas,
        contacts::personas::get_personas_of_user
    ]).launch();
}