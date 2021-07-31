use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError}};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use serde::Serializer;

use crate::{db::{DefaultConnection, Register, schema::info}, impl_register_for, update};

use super::{ContactDescriptor, IsContact};
use crate::db::{Update, Delete};
use diesel::result::Error;

#[derive(Queryable, Insertable, AsChangeset, Deserialize, Clone, Debug)]
#[table_name="info"]
pub struct InfoFragment {
    pub key: String,
    pub value: String,
    pub contact_id: i64
}

impl InfoFragment {

    pub fn new (key: String, value: String, contact_id: i64) -> InfoFragment {
        InfoFragment {
            key,
            value,
            contact_id
        }
    }

    pub fn delete(self, db: &DefaultConnection) -> Result<(), diesel::result::Error> {
        <InfoFragment as Delete>::delete(db, self)?;
        Ok(())
    }

}

impl Delete for InfoFragment {
    type Table = info::table;
    const TABLE: Self::Table = info::table;
    type PrimaryKey = InfoFragment;

    fn delete(db: &DefaultConnection, id: Self::PrimaryKey) -> Result<usize, Error> {
        diesel::delete(info::table.find((id.key, id.value, id.contact_id))).execute(db)
    }
}

impl Serialize for InfoFragment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(1))?;
        s.serialize_entry(&self.key, &self.value)?;
        s.end()
    }
}

impl_register_for!(InfoFragment, InfoFragment, info::table);

#[derive(Clone, Serialize, Deserialize)]
pub struct Info {
    pub contact_id: i64,
    pub info: BareInfo
}

pub type BareInfo = HashMap<String, Vec<String>>;

impl IsContact for Info {
    fn id (&self) -> super::ContactDescriptor {
        ContactDescriptor(self.contact_id)
    }
}

impl Info {

    pub fn of (contact: &(impl IsContact + ?Sized), db: &DefaultConnection) -> Result<Info, diesel::result::Error> {
        let fragments: Vec<InfoFragment> = info::table
            .filter(info::contact_id.eq(contact.id ().0))
            .load::<InfoFragment> (db)?;
        
        let keys = (&fragments).into_iter ()
            .map(|info| info.key.clone ())
            .collect::<HashSet<String>> ();

        let info = keys.into_iter ()
            .map (|key| (key.clone(), (&fragments).into_iter ()
                .filter (|fragment| fragment.key == key)
                .map (|fragment| fragment.value.clone())
                .collect::<Vec<String>> ())
            )
            .collect::<HashMap<String, Vec<String>>> ();
        
        Ok(Info {
            contact_id: contact.id ().0,
            info
        })
    }

    pub fn register(&self, db: &DefaultConnection) -> Result<&Self, diesel::result::Error> {
        for (key, values) in &self.info {
            for value in values {
                let fragment = InfoFragment::new(key.clone (), value.clone(), self.contact_id);
                let result = fragment
                    .register (db);

                match result {
                    Err(DatabaseError(UniqueViolation, _)) => {}
                    _ => { result?; }
                }
            }
        }
        Ok(self)
    }

}

impl<S> std::ops::Index<S> for Info 
    where S: AsRef<str> + ToString
{
    type Output = Vec<String>;

    fn index(&self, key: S) -> &Self::Output {
        &self.info[&key.to_string ()]
    }
}