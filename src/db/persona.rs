use diesel::{QueryDsl, RunQueryDsl};
use crate::{diesel::ExpressionMethods, impl_query_by_id, impl_register_for};
use serde::{Deserialize, Serialize};
use super::{DefaultConnection, contact::Contact, user::User};
use super::schema::*;

#[derive(Clone, Queryable, Serialize, Deserialize, Debug)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub private: bool,
    pub user_id: i64
}

impl_query_by_id!(Persona => personas::table);

#[derive(Clone, Insertable, Serialize, Deserialize, Debug)]
#[table_name="personas"]
pub struct NewPersona {
    pub name: String,
    pub private: bool,
    pub user_id: i64
}

trait IsPersona {

    fn user_id (&self) -> i64;
    fn name (&self) -> &String;

    fn get_user(&self, db: &DefaultConnection) -> Result<User, diesel::result::Error> {
        users::table
            .filter(users::id.eq(self.user_id ()))
            .limit(1)
            .load::<User>(db)
            .map(|users| users[0].clone ())
    }

    fn get_contacts(&self, db: DefaultConnection) -> Result<Vec<(Contact, Persona)>, diesel::result::Error> {
        contacts::table
            .inner_join(personas::table)
            .filter(personas::name.eq(self.name ()))
            .load::<(Contact, Persona)> (&db)
    }

}

impl NewPersona {

    pub fn new(name: String, private: bool, user_id: i64) -> Self {
        Self { name, private, user_id } 
    }

    pub fn new_default(user_id: i64) -> Self {
        Self { 
            name: "default".to_string (),
            private: false,
            user_id
        }
    }

}

macro_rules! impl_is_persona {
    ($self:path) => {
        impl IsPersona for $self {

            fn user_id (&self) -> i64 {
                self.user_id
            }

            fn name (&self) -> &String {
                &self.name
            }

        }
    };
}

impl_register_for!(NewPersona, Persona, personas::table);

impl_is_persona!(Persona);
impl_is_persona!(NewPersona);