use super::{schema::users};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use serde::{Serialize, Deserialize};
use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};
use sha2::{Digest, Sha512};
use crate::{diesel::ExpressionMethods, impl_register_for};

#[derive(Clone, Queryable, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl User {

    pub fn query_by(username: String, db: &PgConnection) -> Result<User, diesel::result::Error> {
        users::table
            .filter (users::username.eq(username))
            .limit(1)
            .load::<User> (db)
            .map(|users| users[0].clone())
    }

}

#[derive(Clone, Insertable, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub trait Password: Clone {
    
    fn password (&self) -> &String;
    fn set_password (&mut self, pass: String);

    fn encrypt (&self) -> Self {
        let mut out = self.clone ();

        for _ in 0..5 {
            let mut engine = Sha512::new();
            engine.update (out.password ());
            let data = engine.finalize();
            out.set_password (base64::encode(&data[..]));
        }

        out
    }

    fn salt (&self) -> Result<Self, BcryptError> {
        match hash (self.password (), DEFAULT_COST) {
            Ok(pass) => { 
                let mut out = self.clone ();
                out.set_password (pass); 
                Ok(out)
            },
            Err(e) => { return Err(e); },
        }
    }

    fn password_cmp<T: Password> (&self, other: &T) -> Result<bool, BcryptError> {
        verify(other.password (), self.password ())
    }

}

#[macro_export]
macro_rules! derive_password {
    ($self:path) => {
        impl Password for $self {
            fn password(&self) -> &String {
                &self.password
            }
            fn set_password(&mut self, pass: String) {
                self.password = pass;
            }
        }
    };
}

impl_register_for!(NewUser, User, users::table);
derive_password!(NewUser);
derive_password!(User);