use super::{DefaultConnection, contact::Contact, schema::{users, contacts, users_contacts_join}};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Serialize, Deserialize};
use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};
use sha2::{Digest, Sha512};
use crate::{diesel::ExpressionMethods, impl_query_by_id, impl_register_for};
use crate::db::Delete;
use crate::update;
use std::marker::PhantomData;
use diesel::result::Error;

#[derive(Clone, Queryable, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub level: i32
}

impl User {

    pub fn query_by_username(username: &String, db: &DefaultConnection) -> Result<User, diesel::result::Error> {
        users::table
            .filter (users::username.eq(username))
            .limit(1)
            .first::<User> (db)
    }

}

impl_query_by_id!(User => users::table);

#[derive(Clone, Insertable, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub level: i32
}

#[derive(Clone, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub level: Option<i32>
}

update! (UpdateUser => NewUser, i64);

impl Delete for ForUser<User> {
    type Table = users::table;
    const TABLE: Self::Table = users::table;
    type PrimaryKey = i64;

    fn delete(&self, db: &DefaultConnection, id: Self::PrimaryKey) -> Result<usize, Error> {
        if self.0 != id {
            return Err(Error::NotFound)
        }
        diesel::delete(users::table)
            .filter(users::id.eq(id))
            .execute(db)
    }
}

impl NewUser {
    pub fn new(username: String, email: String, password: String) -> Self { Self { username, email, password, level: 0 } }
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

#[derive(Copy, Clone, Eq, PartialOrd, PartialEq, Ord)]
pub struct UserId(pub(in crate::db) i64);

impl UserId {
    pub fn new(id: i64) -> UserId {
        UserId(id)
    }
}

impl std::ops::Deref for UserId {
    type Target = i64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Eq, PartialOrd, PartialEq, Ord)]
pub struct ForUser<T>(pub i64, PhantomData<T>);

impl<T> ForUser<T> {
    pub fn into<G>(&self) -> ForUser<G> {
        ForUser(self.0, PhantomData)
    }
}

impl<T> From<UserId> for ForUser<T> {
    fn from(user: UserId) -> Self {
        ForUser (user.0, PhantomData)
    }
}