use diesel::{Connection, Expression, Insertable, QuerySource, RunQueryDsl, Table, insertable::CanInsertInSingleQuery, pg::PgConnection, query_builder::QueryFragment, types::HasSqlType};
use std::env;

pub mod schema;
pub mod user;
pub mod persona;
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

pub trait Register: Insertable<Self::Table> + Sized 
    where 
    <Self::Table as QuerySource>::FromClause: QueryFragment<DefaultBackend>,
    Self::Values: QueryFragment<DefaultBackend> + CanInsertInSingleQuery<DefaultBackend>,
    <Self::Table as Table>::AllColumns: QueryFragment<DefaultBackend>,
    DefaultBackend: HasSqlType<<<Self::Table as Table>::AllColumns as diesel::Expression>::SqlType>
{

    type Table: Table;
    const TABLE: Self::Table;

    type Queryable: diesel::Queryable<<<Self::Table as diesel::Table>::AllColumns as Expression>::SqlType, <DefaultConnection as Connection>::Backend>;

    fn register (self, db: &DefaultConnection) -> Result<Self::Queryable, diesel::result::Error> {
        diesel::insert_into(Self::TABLE)
            .values(self)
            .get_result::<Self::Queryable>(db)
    }
    
}

#[macro_export]
macro_rules! impl_register_for {
    ($self:path, $query_type:path, $table:path) => {
        impl crate::db::Register for $self {
            type Table = $table;
            const TABLE: $table = $table;
            type Queryable = $query_type;
        }
    };
}