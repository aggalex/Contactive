use diesel::{Connection, Expression, Table, pg::PgConnection, result::Error};
use rocket::http::Status;
use std::env;

use crate::routing::ToStatus;
use diesel::query_builder::{AsChangeset};
pub mod schema;
pub mod user;
// pub mod persona;
pub mod contact;

fn establish_connection() -> PgConnection {

    let database_url = env::var("DATABASE_URL")
        .expect("DB_URL must be set");
    let connection = PgConnection::establish(&database_url[..])
        .expect(&format!("Error connecting to {}", database_url));

    connection
}
pub struct DBState (PgConnection);

impl DBState {

    pub fn new () -> DBState {
        DBState(establish_connection())
    }

}

impl std::ops::Deref for DBState {

    type Target = PgConnection;

    fn deref (&self) -> &Self::Target {
        &self.0
    }

}

impl std::ops::DerefMut for DBState {

    fn deref_mut (&mut self) -> &mut Self::Target {
        &mut self.0
    }

}

unsafe impl Send for DBState {

}

unsafe impl Sync for DBState {
    
}

pub type DefaultConnection = PgConnection;
pub type DefaultBackend = <DefaultConnection as Connection>::Backend;

pub trait Register: {

    type Table: Table;
    const TABLE: Self::Table;

    type Queryable: diesel::Queryable<<<Self::Table as diesel::Table>::AllColumns as Expression>::SqlType, <DefaultConnection as Connection>::Backend>;

    fn register (self, db: &DefaultConnection) -> diesel::result::QueryResult<Self::Queryable>;
    
}

pub trait Update: AsChangeset + Sized {

    type Table: Table;
    const TABLE: Self::Table;

    type Queryable;

    type PrimaryKey;

    fn update (&self, db: &DefaultConnection, ids: Self::PrimaryKey) -> Result<Self::Queryable, diesel::result::Error>;
}

pub trait Delete {
    type Table: Table;
    const TABLE: Self::Table;

    type PrimaryKey;

    fn delete (&self, db: &DefaultConnection, id: Self::PrimaryKey) -> Result<usize, diesel::result::Error>;
}

#[macro_export]
macro_rules! delete {
    ($t:ty => $reg:ty, $key:path) => {
        impl crate::db::Delete for $t {
            type Table = <$reg as crate::db::Register>::Table;
            const TABLE: Self::Table = <$reg as crate::db::Register>::TABLE;

            type PrimaryKey = $key;

            fn delete (db: &DefaultConnection, id: Self::PrimaryKey) -> Result<usize, diesel::result::Error> {
                diesel::delete(<Self as crate::db::Delete>::TABLE.find(id)).execute(db)
            }

        }
    }
}

#[macro_export]
macro_rules! update {
    ($t:ty => $reg:ty, $key:ty) => {
        impl crate::db::Update for $t {
            type Table = <$reg as crate::db::Register>::Table;
            const TABLE: Self::Table = <$reg as crate::db::Register>::TABLE;
            type Queryable = <$reg as crate::db::Register>::Queryable;

            // type PrimaryKey = <Self::Table as diesel::query_source::Table>::PrimaryKey;
            type PrimaryKey = $key;

            fn update(&self, db: &DefaultConnection, id: Self::PrimaryKey) -> Result<Self::Queryable, diesel::result::Error> {
                diesel::update(<Self as crate::db::Update>::TABLE.find(id))
                    .set(self)
                    .get_result(db)
            }
        }
    }
}

pub trait QueryById: Sized {

    fn query_by_id (id: i64, db: &DefaultConnection) -> Result<Self, diesel::result::Error>; 

}

#[macro_export]
macro_rules! impl_query_by_id {
    ($self:path => $table:path) => {
        impl crate::db::QueryById for $self {
            fn query_by_id (id: i64, db: &DefaultConnection) -> Result<Self, diesel::result::Error> {
                $table
                    .find (id)
                    .first::<Self> (db)
            }
        }
    };
}

pub trait ConjuctionTable: Register + Sized
{

    type A;
    type B;

    fn get_both(&self, db: &DefaultConnection)
        -> diesel::result::QueryResult<(Self::A, Self::B)>;
    fn check_relation(&self, db: &DefaultConnection) -> diesel::result::QueryResult<Self>;

}

impl ToStatus for Error {
    fn to_status (&self) -> Status {
        match self {
            Error::InvalidCString(_) => Status::InternalServerError,
            Error::DatabaseError(kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => Status::UnprocessableEntity,
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => Status::UnprocessableEntity,
                diesel::result::DatabaseErrorKind::UnableToSendCommand => Status::InternalServerError,
                diesel::result::DatabaseErrorKind::SerializationFailure => Status::InternalServerError,
                diesel::result::DatabaseErrorKind::__Unknown => Status::InternalServerError
            }
            Error::NotFound => Status::NotFound,
            Error::QueryBuilderError(_) => Status::InternalServerError,
            Error::DeserializationError(_) => Status::InternalServerError,
            Error::SerializationError(_) => Status::InternalServerError,
            Error::RollbackTransaction => Status::Ok,
            Error::AlreadyInTransaction => Status::InternalServerError,
            Error::__Nonexhaustive => Status::InternalServerError,
        }
    }
}

#[macro_export]
macro_rules! impl_register_for {
    ($self:path, $query_type:path, $table:path) => {
        impl crate::db::Register for $self {
            type Table = $table;
            const TABLE: $table = $table;
            type Queryable = $query_type;

            fn register(self, db: &DefaultConnection) -> diesel::result::QueryResult<Self::Queryable> {
                diesel::insert_into(<Self as crate::db::Register>::TABLE)
                    .values(self)
                    .get_result::<Self::Queryable>(db)
            }
        }
    };
}