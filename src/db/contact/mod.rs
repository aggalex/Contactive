use diesel::{QueryDsl, Queryable};
use serde::{Deserialize, Serialize};
use crate::impl_register_for;
use crate::diesel::{RunQueryDsl, ExpressionMethods};
use self::info::Info;

use super::{ConjuctionTable, DefaultConnection, QueryById, schema::{contacts, users_contacts_join}, user::{NewUser, User}};
use crate::impl_query_by_id;

pub mod info;

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct NewContact {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub persona: Option<i64>
}

impl_register_for!(NewContact, Contact, contacts::table);

impl NewContact {
    
    pub fn new(name: String, icon: Option<Vec<u8>>, persona: Option<i64>) -> Self { Self { name, icon, persona } }

    pub fn new_default() -> Self {
        Self::new(
            "No Name".to_string (),
            None,
            None
        )
    }

    pub fn new_default_from_persona(persona: i64) -> Self {
        let mut this = Self::new_default ();
        this.persona = Some(persona);
        this
    }

}

#[derive(Queryable, Clone, Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: i64,
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub persona: Option<i64>,
}

impl Contact {

    pub fn of_persona (persona: i64, db: &DefaultConnection) -> Result<Contact, diesel::result::Error> {
        contacts::table
            .filter(contacts::persona.eq (persona))
            .first::<Contact> (db)
    }

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

}

impl IsContact for ContactDescriptor {

    fn id (&self) -> ContactDescriptor {
        self.clone ()
    }

}

impl IsContact for Contact {

    fn id (&self) -> ContactDescriptor {
        ContactDescriptor (self.id)
    }

    fn get_contact(&self, _: &DefaultConnection) -> Result<Contact, diesel::result::Error> {
        Ok(self.clone ())
    }
}

impl_register_for!(UserContactRelation, UserContactRelation, users_contacts_join::table);

#[derive(Clone, Serialize, Deserialize)]
pub struct ContactInfo (pub Contact);