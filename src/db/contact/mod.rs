use diesel::{QueryDsl, Queryable, BoolExpressionMethods, QueryResult, TextExpressionMethods};
use serde::{Deserialize, Serialize};
use crate::impl_register_for;
use crate::diesel::{RunQueryDsl, ExpressionMethods};
use self::info::Info;

use super::{ConjuctionTable, DefaultConnection, schema::{contacts, users_contacts_join}, user::User};
use crate::update;
use crate::db::user::ForUser;
use diesel::result::Error;
use crate::db::schema::{users, search_sort, lower};
use crate::db::{Delete, Register};
use diesel::expression::count::count_star;

pub mod info;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Visibility {
    Local, Private, Public
}

impl From<Visibility> for i16 {
    fn from(v: Visibility) -> Self {
        match v {
            Visibility::Local => 0,
            Visibility::Private => 1,
            Visibility::Public => 2
        }
    }
}

impl From<i16> for Visibility {
    fn from(i: i16) -> Self {
        match i {
            0 => Visibility::Local,
            1 => Visibility::Private,
            _ => Visibility::Public
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostContact {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    visibility: i16,
}

impl ForUser<PostContact> {
    pub fn relate(&self, this: PostContact) -> NewContact {
        self.into::<NewContact>().new(
            this.name,
            this.icon,
            this.visibility
        )
    }
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct NewContact {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    visibility: i16,
    pub creator: i64
}

impl Register for NewContact {
    type Table = contacts::table;
    const TABLE: Self::Table = contacts::table;
    type Queryable = Contact;

    fn register(self, db: &DefaultConnection) -> QueryResult<Self::Queryable> {
        let out = diesel::insert_into(<Self as crate::db::Register>::TABLE)
            .values(self)
            .get_result::<Self::Queryable>(db)?;

        UserContactRelation (
            out.creator,
            out.id
        ).register(db)?;

        Ok(out)
    }
}

impl NewContact {

    pub fn visibility(&self) -> Visibility {
        self.visibility.into()
    }
    pub fn set_visibility(&mut self, v: Visibility) {
        self.visibility = v.into()
    }

}

impl ForUser<NewContact> {
    pub fn new(&self, name: String, icon: Option<Vec<u8>>, vis: impl Into<i16>) -> NewContact {
        NewContact { name, icon,
            visibility: vis.into(),
            creator: self.0
        }
    }

    pub fn new_default(&self) -> NewContact {
        self.new(
            "No Name".to_string (),
            None,
            Visibility::Local
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateContact {
    pub name: Option<String>,
    pub icon: Option<Option<Vec<u8>>>,
    visibility: Option<i16>,
}

#[derive(AsChangeset, Serialize, Deserialize, Clone, Debug)]
#[table_name="contacts"]
pub struct _UpdateContact {
    pub name: Option<String>,
    pub icon: Option<Option<Vec<u8>>>,
    visibility: Option<i16>,
    pub creator: i64
}

impl ForUser<UpdateContact> {
    pub fn get(&self, u: UpdateContact) -> _UpdateContact {
        match u {
            UpdateContact {
                name,
                icon,
                visibility: vis
            } => _UpdateContact {
                name,
                icon,
                visibility: vis,
                creator: self.0
            }
        }
    }
}

impl UpdateContact {
    pub fn visibility(&self) -> Option<Visibility> {
        self.visibility.map(|v| v.into())
    }
    pub fn set_visibility(&mut self, v: Option<Visibility>) {
        self.visibility = v.map(|v| v.into())
    }
}

update!(_UpdateContact => NewContact, i64);
// delete!(Contact => NewContact, i64);

impl Delete for ForUser<Contact> {
    type Table = contacts::table;
    const TABLE: Self::Table = contacts::table;
    type PrimaryKey = i64;

    fn delete(&self, db: &DefaultConnection, id: Self::PrimaryKey) -> Result<usize, Error> {
        self.has_jurisdiction(id, db)?;
        diesel::delete(contacts::table)
            .filter(contacts::id.eq(id))
            .execute(db)
    }
}

#[derive(Queryable, Clone, Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: i64,
    pub name: String,
    pub icon: Option<Vec<u8>>,
    visibility: i16,
    pub creator: i64
}

#[derive(Queryable, Serialize)]
pub struct SearchResults {
    pages: i64,
    contacts: Vec<Contact>
}

impl Contact {
    pub fn force_get_by_id(id: i64, db: &DefaultConnection) -> diesel::result::QueryResult<Contact> {
        contacts::table.filter(contacts::id.eq(id))
            .first::<Contact> (db)
    }

    pub fn search_public(db: &DefaultConnection, page: i64, buffer: i64, query: String) -> diesel::result::QueryResult<SearchResults> {
        let q = contacts::table.filter(
                contacts::visibility.ge(2)
                    .and(lower(contacts::name).like(format!("%{}%", query.to_lowercase()))))
            .order(search_sort(contacts::name, query))
            .then_order_by(contacts::name.asc())
            .offset(page * buffer)
            .limit(buffer);
        Ok(SearchResults {
            pages: q.clone().select(count_star()).first::<i64>(db)?,
            contacts: q.load::<Contact>(db)?
        })
    }
}

impl ForUser<Contact> {
    pub fn query_by_id (&self, id: i64, db: &DefaultConnection) -> diesel::result::QueryResult<Contact> {
        let contact = contacts::table.filter(contacts::id.eq(id))
            .first::<Contact> (db)?;

        self.has_jurisdiction(contact.id, db)?;

        Ok(contact)
    }

    pub fn has_jurisdiction (&self, id: i64, db: &DefaultConnection) -> diesel::result::QueryResult<UserContactRelation> {
        if Contact::force_get_by_id(id, db)?.visibility() == Visibility::Public {
            return Ok(UserContactRelation (
                self.0,
                id
            ))
        }
        users_contacts_join::table.filter(users_contacts_join::user_id.eq(self.0)
            .and(users_contacts_join::contact_id.eq(id)))
            .first::<UserContactRelation>(db)
    }
}

// impl_query_by_id!(Contact => contacts::table);

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Copy, Debug)]
#[table_name="users_contacts_join"]
pub struct UserContactRelation(
    #[column_name = "user_id"]
    pub i64, 
    #[column_name = "contact_id"]
    pub i64
);

impl ConjuctionTable for UserContactRelation {
    
    type A = User;

    type B = Contact;

    fn get_both(&self, db: &DefaultConnection) -> QueryResult<(User, Contact)> {
        Ok((
            users::table.filter(users::id.eq(self.0)).first::<User>(db)?,
            contacts::table.filter(contacts::id.eq(self.1)).first::<Contact>(db)?
            ))
    }

    fn check_relation(&self, db: &DefaultConnection) -> QueryResult<Self> {
        users_contacts_join::table.filter(
            users_contacts_join::user_id.eq(self.0)
                .and(users_contacts_join::contact_id.eq(self.1))
        ).first::<UserContactRelation>(db)
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
        users::table
            .filter(users::id.eq(self.creator()))
            .first::<User> (db)
    }


    fn creator(&self) -> i64;
    fn visibility(&self) -> Visibility;
    fn set_visibility(&mut self, v: Visibility);
}

impl IsContact for ContactDescriptor {

    fn id (&self) -> ContactDescriptor {
        self.clone ()
    }
    fn creator(&self) -> i64 {
        eprintln!("Invalid getter: ContactDescriptor.creator at src/db/contact/mod.rs:{}", line!());
        -1
    }
    fn visibility(&self) -> Visibility {
        eprintln!("Invalid getter: ContactDescriptor.visibility at src/db/contact/mod.rs:{}", line!());
        Visibility::Local
    }
    fn set_visibility(&mut self, _: Visibility) {
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

    fn creator(&self) -> i64 {
        self.creator
    }

    fn visibility(&self) -> Visibility {
        self.visibility.into()
    }

    fn set_visibility(&mut self, v: Visibility) {
        self.visibility = v.into()
    }
}

impl_register_for!(UserContactRelation, UserContactRelation, users_contacts_join::table);

#[derive(Clone, Serialize, Deserialize)]
pub struct ContactInfo (pub Contact);