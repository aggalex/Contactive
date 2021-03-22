use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use crate::{diesel::ExpressionMethods, impl_register_for};
use serde::{Deserialize, Serialize};
use super::{user::User};
use super::schema::personas;
use super::schema::users;

#[derive(Clone, Queryable, Serialize, Deserialize, Debug)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub private: bool,
    pub user_id: i64
}

#[derive(Clone, Insertable, Serialize, Deserialize, Debug)]
#[table_name="personas"]
pub struct NewPersona {
    pub name: String,
    pub private: bool,
    pub user_id: i64
}

trait IsPersona {

    fn get_user_id (&self) -> i64;

    fn get_user(&self, db: &PgConnection) -> Result<User, diesel::result::Error> {
        users::table
            .filter(users::id.eq(self.get_user_id ()))
            .limit(1)
            .load::<User>(db)
            .map(|users| users[0].clone ())
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

impl_register_for!(NewPersona, Persona, personas::table);

macro_rules! impl_is_persona {
    ($self:path) => {
        impl IsPersona for $self {

            fn get_user_id (&self) -> i64 {
                self.user_id
            }

        }
    };
}

impl_is_persona!(Persona);
impl_is_persona!(NewPersona);