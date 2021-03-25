use super::{DefaultConnection, contact::Contact, persona::Persona, schema::{users, contacts, personas, users_contacts_join}};
use diesel::{QueryDsl, RunQueryDsl};
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

    pub fn query_by_username(username: &String, db: &DefaultConnection) -> Result<User, diesel::result::Error> {
        users::table
            .filter (users::username.eq(username))
            .limit(1)
            .first::<User> (db)
    }

    pub fn query_by_id(id: i64, db: &DefaultConnection) -> Result<User, diesel::result::Error> {
        users::table
            .find (id)
            .first::<User> (db)
    }

}

#[derive(Clone, Insertable, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl NewUser {
    pub fn new(username: String, email: String, password: String) -> Self { Self { username, email, password } }
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

pub trait IsUser {

    fn id (&self) -> i64;

    fn get_personas (&self, db: &DefaultConnection) -> Result<Vec<(Contact, Persona)>, diesel::result::Error>;

    fn get_public_personas (&self, db: &DefaultConnection) -> Result<Vec<(Contact, Persona)>, diesel::result::Error> {
        Ok (
            self.get_personas (db)?
                .into_iter ()
                .filter (|tuple| !tuple.1.private)
                .collect::<Vec<(Contact, Persona)>> ()
        )
    }

    fn get_contacts (&self, db: &DefaultConnection) -> Result<Vec<Contact>, diesel::result::Error>;

    fn delete (&self, db: &DefaultConnection) -> Result<usize, diesel::result::Error>;

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

pub trait DBUser {

    fn id (&self) -> i64;

}

impl<U: DBUser> IsUser for U {


    fn id (&self) -> i64 {
        self.id ()
    }

    fn get_personas (&self, db: &DefaultConnection) -> Result<Vec<(Contact, Persona)>, diesel::result::Error>  {
        contacts::table
            .inner_join(personas::table)
            .filter(personas::user_id.eq(self.id ()))
            .load::<(Contact, Persona)> (db)
    }

    fn get_contacts (&self, db: &DefaultConnection) -> Result<Vec<Contact>, diesel::result::Error> {
        Ok (
            users_contacts_join::table
                .filter (users_contacts_join::user_id.eq(self.id ()))
                .inner_join (contacts::table)
                .load::<((i64, i64), Contact)> (db)?
                .into_iter ()
                .map (|descriptor| descriptor.1)
                .collect::<Vec<Contact>> ()
        )
    }

    fn delete(&self, db: &DefaultConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(
            users::table
                .find(self.id ())
        ).execute(db)
    }

}

impl DBUser for User {

    fn id (&self) -> i64 {
        self.id
    }

}

pub struct UserDescriptor(pub i64);

impl DBUser for UserDescriptor {

    fn id (&self) -> i64 {
        self.0
    }


}