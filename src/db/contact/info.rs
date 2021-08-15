use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError}, BoolExpressionMethods};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use serde::Serializer;

use crate::{db::{DefaultConnection, Register, schema::info}, impl_register_for};

use super::{IsContact};
use crate::db::{Delete, ConjuctionTable};
use diesel::result::Error;
use crate::db::schema::info::dsl::key;
use crate::db::schema::info::columns::contact_id;
use crate::db::user::{ForUser, UserId};
use crate::db::contact::{Contact, UserContactRelation};

#[derive(Queryable, Insertable, AsChangeset, Deserialize, Clone, Debug)]
#[table_name="info"]
pub struct InfoFragment {
    pub key: String,
    pub value: String,
    pub contact_id: i64
}

impl InfoFragment {

    pub fn new (other_key: String, value: String, contact: i64) -> InfoFragment {
        InfoFragment {
            key: other_key,
            value,
            contact_id: contact
        }
    }

}

#[derive(Clone)]
pub struct InfoSection {
    pub name: String,
    pub contact: i64
}

impl InfoSection {
    pub fn delete(self, db: &DefaultConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(info::table.filter(key.eq(self.name).and(contact_id.eq(self.contact))))
            .execute(db)
    }
}

pub trait ForContact {
    fn contact_id(&self) -> i64;
}

impl ForContact for InfoFragment {
    fn contact_id(&self) -> i64 {
        self.contact_id
    }
}

impl ForContact for InfoSection {
    fn contact_id(&self) -> i64 {
        self.contact
    }
}

pub struct Jurisdiction<V: ForContact> (Vec<V>);

impl<V: ForContact> Jurisdiction<V> {
    pub fn new (user: UserId, items: Vec<V>, db: &DefaultConnection) -> diesel::result::QueryResult<Jurisdiction<V>> {
        let mut contacts = HashSet::<i64>::new();
        Ok(Jurisdiction(items.into_iter()
            .map(|item| {
                let contact = item.contact_id();
                if contacts.contains(&contact) {
                    Ok(item)
                } else {
                    UserContactRelation(user.0, contact).check_relation(db)?;
                    contacts.insert(contact);
                    Ok(item)
                }
            })
            .collect::<diesel::result::QueryResult<Vec<V>>>()?))
    }
}

impl Delete for Jurisdiction<InfoFragment> {
    type Table = info::table;
    const TABLE: Self::Table = info::table;
    type PrimaryKey = ();

    fn delete(&self, db: &DefaultConnection, _: ()) -> Result<usize, Error> {
        let (keyvals, contact_ids): (Vec<(String, String)>, Vec<i64>) = self.0.clone().into_iter()
            .map(|v| ((v.key, v.value), v.contact_id))
            .unzip();
        let (keys, values): (Vec<String>, Vec<String>) = keyvals.into_iter()
            .unzip();
        diesel::delete(info::table).filter(
            info::key.eq_any(keys)
                .and(info::value.eq_any(values))
                .and(info::contact_id.eq_any(contact_ids)))
            .execute(db)
    }
}

impl Delete for Jurisdiction<InfoSection> {
    type Table = info::table;
    const TABLE: Self::Table = info::table;
    type PrimaryKey = ();

    fn delete(&self, db: &DefaultConnection, _: ()) -> Result<usize, Error> {
        let (contacts, names): (Vec<i64>, Vec<String>) = self.0.clone().into_iter()
            .map(|v| (v.contact, v.name))
            .unzip();
        diesel::delete(info::table).filter(
            info::key.eq_any(names)
                .and(info::contact_id.eq_any(contacts))
        ).execute(db)
    }
}

impl Delete for ForUser<InfoFragment> {
    type Table = info::table;
    const TABLE: Self::Table = info::table;
    type PrimaryKey = InfoFragment;

    fn delete(&self, db: &DefaultConnection, framgent: Self::PrimaryKey) -> Result<usize, Error> {
        self.into::<Contact>().has_jurisdiction(framgent.contact_id, db)?;
        diesel::delete(info::table.find((framgent.key, framgent.value, framgent.contact_id))).execute(db)
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
pub struct FullInfo {
    pub contact: Contact,
    pub info: BareInfo
}

impl FullInfo {
    pub fn of (contact: Contact, db: &DefaultConnection) -> Result<FullInfo, diesel::result::Error> {
        Ok(FullInfo {
            info: Info::of(&contact, db)?.info,
            contact,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Info {
    pub contact_id: i64,
    pub info: BareInfo
}

pub type BareInfo = HashMap<String, Vec<String>>;

impl Info {

    pub fn of (contact: &(impl IsContact + ?Sized), db: &DefaultConnection) -> Result<Info, diesel::result::Error> {
        let fragments: Vec<InfoFragment> = info::table
            .filter(info::contact_id.eq(contact.id ().0))
            .load::<InfoFragment> (db)?;
        
        let keys = (&fragments).into_iter ()
            .map(|info| info.key.clone ())
            .collect::<HashSet<String>> ();

        let info = keys.into_iter ()
            .map (|k| (k.clone(), (&fragments).into_iter ()
                .filter (|fragment| fragment.key == k)
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
        for (k, values) in &self.info {
            for value in values {
                let fragment = InfoFragment::new(k.clone (), value.clone(), self.contact_id);
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

    fn index(&self, k: S) -> &Self::Output {
        &self.info[&k.to_string ()]
    }
}