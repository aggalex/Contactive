use diesel::{QueryDsl, Queryable};
use serde::{Deserialize, Serialize};
use crate::impl_register_for;
use crate::diesel::{RunQueryDsl, ExpressionMethods};
use self::info::Info;

use super::{ConjuctionTable, DefaultConnection, QueryById, schema::{contacts, users_contacts_join}, user::{NewUser, User}, Register, Update};
use crate::{impl_query_by_id, update, delete};
use rocket::logger::warn;

pub mod info;

#[derive(Copy, Clone)]
pub enum Visibilty {
    Local, Private, Public
}

impl From<Visibilty> for i16 {
    fn from(v: Visibilty) -> Self {
        match v {
            Visibilty::Local => 0,
            Visibilty::Private => 1,
            Visibilty::Public => 2
        }
    }
}

impl From<i16> for Visibilty {
    fn from(i: i16) -> Self {
        match i {
            0 => Visibilty::Local,
            1 => Visibilty::Private,
            _ => Visibilty::Public
        }
    }
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct NewContact {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    visibility: i16
}

impl_register_for!(NewContact, Contact, contacts::table);

impl NewContact {
    
    pub fn new(name: String, icon: Option<Vec<u8>>, visibility: Visibilty) -> Self {
        Self { name, icon,
            visibility: visibility.into()
        }
    }

    pub fn new_default() -> Self {
        Self::new(
            "No Name".to_string (),
            None,
            Visibilty::Local.into()
        )
    }

    fn visibility(&self) -> Visibilty {
        self.visibility.into()
    }
    fn set_visibility(&mut self, v: Visibilty) {
        self.visibility = v.into()
    }

}

#[derive(AsChangeset, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct UpdateContact {
    pub name: Option<String>,
    pub icon: Option<Option<Vec<u8>>>,
    visibility: Option<i16>
}

impl UpdateContact {
    fn visibility(&self) -> Option<Visibilty> {
        self.visibility.map(|v| v.into())
    }
    fn set_visibility(&mut self, v: Option<Visibilty>) {
        self.visibility = v.map(|v| v.into())
    }
}

update!(UpdateContact => NewContact, i64);
delete!(Contact => NewContact, i64);

#[derive(Queryable, Clone, Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: i64,
    pub name: String,
    pub icon: Option<Vec<u8>>,
    visibility: i16
}

impl_query_by_id!(Contact => contacts::table);

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Copy, Debug)]
#[table_name="users_contacts_join"]
pub struct UserContactRelation(
    #[column_name = "user_id"]
    pub i64, 
    #[column_name = "contact_id"]
    pub i64
);

impl ConjuctionTable for UserContactRelation {
    
    type A = NewUser;

    type B = NewContact;

    fn as_tuple (&self) -> (i64, i64) {
        (self.0, self.1)
    }
}

#[derive(Clone, Copy)]
pub struct ContactDescriptor(pub i64);

pub trait IsContact {

    fn id (&self) -> ContactDescriptor;

    fn get_contact (&self, db: &DefaultConnection) -> Result<Contact, diesel::result::Error> {
        contacts::table
            .find (self.id ().0)
            .first::<Contact> (db)
    }

    fn get_all_info (&self, db: &DefaultConnection) -> Result<Info, diesel::result::Error> {
        Info::of (self, db)
    }

    fn get_user (&self, db: &DefaultConnection) -> Result<User, diesel::result::Error> {
        let id = users_contacts_join::table
            .filter(users_contacts_join::contact_id.eq(self.id().0))
            .first::<UserContactRelation> (db)?
            .0;
        
        User::query_by_id(id, db)
    }

    fn visibility(&self) -> Visibilty;
    fn set_visibility(&mut self, v: Visibilty);
}

impl IsContact for ContactDescriptor {

    fn id (&self) -> ContactDescriptor {
        self.clone ()
    }
    fn visibility(&self) -> Visibilty {
        eprintln!("Invalid getter: ContactDescriptor.visibility at src/db/contact/mod.rs:{}", line!());
        Visibilty::Local
    }
    fn set_visibility(&mut self, _: Visibilty) {
        eprintln!("Invalid setter: ContactDescriptor.set_visibility at src/db/contact/mod.rs:{}", line!());
    }
}

impl IsContact for Contact {

    fn id (&self) -> ContactDescriptor {
        ContactDescriptor (self.id)
    }

    fn get_contact(&self, _: &DefaultConnection) -> Result<Contact, diesel::result::Error> {
        Ok(self.clone ())
    }

    fn visibility(&self) -> Visibilty {
        self.visibility.into()
    }

    fn set_visibility(&mut self, v: Visibilty) {
        self.visibility = v.into()
    }
}

impl_register_for!(UserContactRelation, UserContactRelation, users_contacts_join::table);

#[derive(Clone, Serialize, Deserialize)]
pub struct ContactInfo (pub Contact);