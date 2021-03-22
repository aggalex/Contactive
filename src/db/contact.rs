use serde::{Deserialize, Serialize};
use crate::impl_register_for;

use super::schema::{contacts, users_contacts_join};
use chrono::NaiveDate;

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct NewContact {
    pub name: String,
    pub birthday: Option<NaiveDate>,
    pub icon: Option<Vec<u8>>,
    pub persona: Option<i64>
}

impl_register_for!(NewContact, Contact, contacts::table);

impl NewContact {
    
    pub fn new(name: String, birthday: Option<NaiveDate>, icon: Option<Vec<u8>>, persona: Option<i64>) -> Self { Self { name, birthday, icon, persona } }

    pub fn new_default() -> Self {
        Self::new(
            "No Name".to_string (),
            None,
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
    pub birthday: Option<NaiveDate>,
    pub icon: Option<Vec<u8>>,
    pub persona: Option<i64>,
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Copy, Debug)]
#[table_name="users_contacts_join"]
pub struct UserContactRelation(
    #[column_name = "user_id"]
    pub i64, 
    #[column_name = "contact_id"]
    pub i64
);

impl_register_for!(UserContactRelation, UserContactRelation, users_contacts_join::table);