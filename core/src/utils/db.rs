use alohomora::bbox::BBox;

use entity::{admin_session, application};
use portfolio_policies::data::CandidateDataPolicy;
use sea_orm::{DbConn, Statement};
use portfolio_policies::FakePolicy;

use crate::Query;
use crate::error::ServiceError;

#[cfg(test)]
pub const TESTING_ADMIN_COOKIE2: &str = "0xdeadbeef12345678deadbeef12345678";

pub const TESTING_ADMIN_COOKIE: &str = "0xdeadbeef12345678deadbeef12345678";
const TESTING_ADMIN_ID: &str = "1";
pub const TESTING_ADMIN_KEY: &str = "blahblah";

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


    //let base_url = String::from("sqlite::memory:");
    let database_url = std::env::var("PORTFOLIO_DATABASE_URL").unwrap();//.unwrap_or(base_url);
    println!("TRYING TO CONNECT TO {}", database_url);
    let db: DbConn = Database::connect(database_url.clone()).await.unwrap();

    // make testing db (if doesn't already exist)
    let db_name = "portfolio_test";
    let _ = db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("DROP DATABASE `{}`;", db_name),
    )).await;
    let _ = db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
    )).await;
    
    // then connect directy to testing db
    let db: DbConn = Database::connect(
        format!("{database_url}{db_name}")
    ).await.unwrap();
    
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

    // switch everything from varchars to text
    let query = "ALTER TABLE candidate MODIFY COLUMN name text; ALTER TABLE candidate MODIFY COLUMN surname text; ALTER TABLE candidate MODIFY COLUMN birth_surname text; ALTER TABLE candidate MODIFY COLUMN birthplace text; ALTER TABLE candidate MODIFY COLUMN address text; ALTER TABLE candidate MODIFY COLUMN letter_address text; ALTER TABLE candidate MODIFY COLUMN telephone text; ALTER TABLE candidate MODIFY COLUMN citizenship text; ALTER TABLE candidate MODIFY COLUMN email text; ALTER TABLE candidate MODIFY COLUMN sex text; ALTER TABLE candidate MODIFY COLUMN school_name text; ALTER TABLE candidate MODIFY COLUMN personal_identification_number text; ALTER TABLE candidate MODIFY COLUMN health_insurance text; ALTER TABLE candidate MODIFY COLUMN grades_json text; ALTER TABLE candidate MODIFY COLUMN first_school text; ALTER TABLE candidate MODIFY COLUMN second_school text; ALTER TABLE candidate MODIFY COLUMN test_language text; ALTER TABLE parent MODIFY COLUMN name text; ALTER TABLE parent MODIFY COLUMN surname text; ALTER TABLE parent MODIFY COLUMN telephone text; ALTER TABLE parent MODIFY COLUMN email text; ALTER TABLE application MODIFY COLUMN personal_id_number text; ALTER TABLE candidate MODIFY COLUMN birthdate text;".to_string();
    for a in query.split("; "){
        println!("executing {}", a);
        let _ = db.execute(Statement::from_string(
            db.get_database_backend(),
            a.to_string(),
        )).await;
    }

    // add a admin session for testing
    let _ = db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO admin_session 
            (id, admin_id, ip_address, created_at, expires_at, updated_at) VALUES 
            (".to_string() + TESTING_ADMIN_COOKIE + ", "
                + TESTING_ADMIN_ID + ", \"127.0.0.1\", 
                NOW(), NOW() + 10000000000000, NOW());",
//              ^now   ^session expiry date    ^updated date
    )).await;

    db
}

#[cfg(test)]
use portfolio_api::pool::ContextDataType;
use std::marker::PhantomData;
use alohomora::context::{Context, ContextData};
use alohomora::testing::TestContextData;
use alohomora::policy::NoPolicy;

#[cfg(test)]
pub async fn get_test_context(db: &DbConn) -> Context<TestContextData<ContextDataType>> {
    Context::test(ContextDataType{
        session_id: Some(BBox::new(TESTING_ADMIN_COOKIE.to_string(), NoPolicy::new())),
        key: Some(BBox::new(TESTING_ADMIN_KEY.to_string(), NoPolicy::new())),
        conn: unsafe{ std::mem::transmute(db)},
        candidate_login: None,
        admin_login: None,
        phantom: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::get_memory_sqlite_connection;

    #[tokio::test]
    async fn test_get_memory_sqlite_connection() {
        get_memory_sqlite_connection().await;  
    }
}