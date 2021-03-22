use diesel::{Connection, Expression, Insertable, RunQueryDsl, Table, backend::SupportsReturningClause, insertable::CanInsertInSingleQuery, pg::PgConnection, query_builder::QueryFragment, types::HasSqlType};
use std::env;

pub mod schema;
pub mod user;
pub mod persona;
pub mod contact;

fn establish_connection() -> PgConnection {

    let database_url = env::var("DB_URL")
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

pub trait Register<T: Table, DB: Connection = PgConnection>: Insertable<T> + Sized
    where
        T::FromClause: QueryFragment<DB::Backend>,
        Self::Values: QueryFragment<DB::Backend> + CanInsertInSingleQuery<DB::Backend>,
        DB::Backend: SupportsReturningClause,
        T::AllColumns: QueryFragment<DB::Backend>,
        DB::Backend: HasSqlType<<<T as diesel::Table>::AllColumns as diesel::Expression>::SqlType>
{

    const TABLE: T;

    type Queryable: diesel::Queryable<<<T as diesel::Table>::AllColumns as Expression>::SqlType, DB::Backend>;

    fn register (self, db: &DB) -> Result<Self::Queryable, diesel::result::Error> {
        diesel::insert_into(Self::TABLE)
            .values(self)
            .get_result::<Self::Queryable>(db)
    }
    
}

#[macro_export]
macro_rules! impl_register_for {
    ($self:path, $query_type:path, $table:path) => {
        impl crate::db::Register<$table> for $self {
            const TABLE: $table = $table;
            type Queryable = $query_type;
        }
    };
}