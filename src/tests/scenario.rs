use rocket::{http::{ContentType, Status}, local::Client};
use crate::{db::user::NewUser, routing::user::Login};

pub type AnyError = Box<dyn std::error::Error + 'static>;

#[allow(dead_code)]
pub fn test_user () -> NewUser {
    NewUser::new (
        "aggelalex".to_string (),
        "ubuntu1aggelalex@gmail.com".to_string (),
        "yo mama bin and fat".to_string ()
    )
}

#[derive(Clone, Debug)]
pub struct StatusError (Status);

impl std::error::Error for StatusError {}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

trait StatusErrorCheck {

    fn check (&self) -> Result<(), StatusError>;

}

impl StatusErrorCheck for Status {

    fn check (&self) -> Result<(), StatusError> {
        if self != &Status::Ok {
            return Err(StatusError(self.clone()))
        }
        Ok(())
    }

}

pub trait ClientActions {
    fn register_user (&self, user: &NewUser) -> Result<(), AnyError>;
    fn login (&self, user: &Login) -> Result<(), AnyError>;
    fn delete_user (&self) -> Result<(), AnyError>;
    fn add_contacts (&self) -> Result<(), AnyError>;
}

impl ClientActions for Client {
    fn register_user (&self, user: &NewUser) -> Result<(), AnyError> {
        Ok(self
            .post("/register")
            .body(serde_json::to_string (user)?)
            .header(ContentType::JSON)
            .dispatch()
            .status()
            .check()?)
    }

    fn login (&self, user: &Login) -> Result<(), AnyError> {
        Ok(self
            .post("/login")
            .body(serde_json::to_string (user)?)
            .header(ContentType::JSON)
            .dispatch()
            .status()
            .check()?)
    }

    fn delete_user (&self) -> Result<(), AnyError> {
        Ok(self
            .delete("/")
            .dispatch()
            .status()
            .check()?)
    }

    fn add_contacts (&self) -> Result<(), AnyError> {
        todo!()
    }
}