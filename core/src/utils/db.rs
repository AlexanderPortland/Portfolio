use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use rocket::shield::Policy;
use entity::{admin_session, application};
use sea_orm::{DbConn, Statement};

use crate::Query;
use crate::error::ServiceError;

pub async fn get_recipients(db: &DbConn, candidate_pubkey: BBox<String, NoPolicy>)
    -> Result<Vec<BBox<String, NoPolicy>>, ServiceError>
{
    let mut admin_public_keys = Query::get_all_admin_public_keys_together(db).await?;
    admin_public_keys.push(candidate_pubkey);
    Ok(admin_public_keys)
}

pub async fn get_memory_sqlite_connection() -> sea_orm::DbConn {
    use entity::{admin, candidate, parent, session};
    use sea_orm::{Schema, Database};
    use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, DbBackend};


    let base_url = String::from("sqlite::memory:");
    let database_url = std::env::var("PORTFOLIO_DATABASE_URL").unwrap_or(base_url);
    println!("TRYING TO CONNECT TO {}", database_url);
    let db: DbConn = Database::connect(database_url.clone()).await.unwrap();
    
    let schema = Schema::new(DbBackend::Sqlite);
    let stmt: TableCreateStatement = schema.create_table_from_entity(candidate::Entity);
    let stmt2: TableCreateStatement = schema.create_table_from_entity(application::Entity);
    let stmt3: TableCreateStatement = schema.create_table_from_entity(session::Entity);
    let stmt4: TableCreateStatement = schema.create_table_from_entity(admin::Entity);
    let stmt5: TableCreateStatement = schema.create_table_from_entity(admin_session::Entity);
    let stmt6: TableCreateStatement = schema.create_table_from_entity(parent::Entity);
    db.execute(db.get_database_backend().build(&stmt)).await.unwrap();
    db.execute(db.get_database_backend().build(&stmt2)).await.unwrap();
    db.execute(db.get_database_backend().build(&stmt3)).await.unwrap();
    db.execute(db.get_database_backend().build(&stmt4)).await.unwrap();
    db.execute(db.get_database_backend().build(&stmt5)).await.unwrap();
    db.execute(db.get_database_backend().build(&stmt6)).await.unwrap();

    db
}

#[cfg(test)]
mod tests {
    use super::get_memory_sqlite_connection;

    #[tokio::test]
    async fn test_get_memory_sqlite_connection() {
        get_memory_sqlite_connection().await;  
    }
}