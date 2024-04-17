use async_trait::async_trait;
use portfolio_core::sea_orm::{self};
#[cfg(not(test))]
use sea_orm::ConnectOptions;
use alohomora::orm::Database;
#[cfg(not(test))]
use std::time::Duration;

use rocket::figment::Figment;

#[derive(Database, Debug)]
#[database("sea_orm")]
pub struct Db(SeaOrmPool);

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl alohomora::orm::Pool for SeaOrmPool {
    type Error = sea_orm::DbErr;

    type Connection = sea_orm::DatabaseConnection;

    #[cfg(test)]
    async fn init(_figment: &Figment) -> Result<Self, Self::Error> {

        let conn = portfolio_core::utils::db::get_memory_sqlite_connection().await;
        crate::test::tests::run_test_migrations(&conn).await;
        return Ok(Self { conn });
    }

    #[cfg(not(test))]
    async fn init(_figment: &Figment) -> Result<Self, Self::Error> {
        let init = false;

        dotenv::dotenv().ok();
        println!("NO TEST");

        let database_url = std::env::var("PORTFOLIO_DATABASE_URL").unwrap();
        let mut options: ConnectOptions = database_url.clone().into();
        options
            .max_connections(1024)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(15))
            .acquire_timeout(Duration::from_secs(15))
            .max_lifetime(Duration::from_secs(15))
            .idle_timeout(Duration::from_secs(5))
            .sqlx_logging(false);

        /* options
        .max_connections(config.max_connections as u32)
        .min_connections(config.min_connections.unwrap_or_default())
        .connect_timeout(Duration::from_secs(config.connect_timeout));
        if let Some(idle_timeout) = config.idle_timeout {
            options.idle_timeout(Duration::from_secs(idle_timeout));
        } */
        println!("connecting");

        // connect to general database
        let db: sea_orm::DbConn = sea_orm::Database::connect(options).await?;

        use portfolio_core::crypto::{self, hash_password};
        use sea_orm::{Schema, Statement};
        use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, DbBackend};

        // create specific portfolio db if it doesn't exist
        let db_name = "portfolio";
        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
        )).await?;

        // connect directly to that one
        let mut options2: ConnectOptions = format!("{database_url}{db_name}").clone().into();
        options2
            .max_connections(1024)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(15))
            .acquire_timeout(Duration::from_secs(15))
            .max_lifetime(Duration::from_secs(15))
            .idle_timeout(Duration::from_secs(5))
            .sqlx_logging(false);
        let db = sea_orm::Database::connect(options2).await?;
        println!("CONNECTED to new");

        
        if init {
            use entity::{admin, candidate, parent, session};

            // make schema from all our entities
            let schema = Schema::new(DbBackend::MySql);
            let stmt: TableCreateStatement = schema.create_table_from_entity(candidate::Entity);
            let stmt2: TableCreateStatement = schema.create_table_from_entity(entity::application::Entity);
            let stmt3: TableCreateStatement = schema.create_table_from_entity(session::Entity);
            let stmt4: TableCreateStatement = schema.create_table_from_entity(admin::Entity);
            let stmt5: TableCreateStatement = schema.create_table_from_entity(entity::admin_session::Entity);
            let stmt6: TableCreateStatement = schema.create_table_from_entity(parent::Entity);
            
            // build db based on that schema
            let b = db.get_database_backend().build(&stmt);
            let r = db.execute(b).await;
            r.unwrap();
            db.execute(db.get_database_backend().build(&stmt2)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt3)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt4)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt5)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt6)).await.unwrap();
        }

        if true {
            // insert new admin account for our use
            // id: should be 1 but may change
            // password: hello

            let password_plain_text = "hello".to_string();
            let pwrd_hash = hash_password(password_plain_text.clone()).await.unwrap();
            println!("got password hash {}", pwrd_hash.clone());
            let (pub_key, priv_key) = crypto::create_identity();
            let enc_priv_key = crypto::encrypt_password(priv_key.discard_box(), password_plain_text).await.unwrap();
            // db.execute(Statement::from_string(
            //     db.get_database_backend(),
            //     format!("INSERT INTO admin VALUES (0, \"alex3\", \"{pub_key}\", \"{enc_priv_key}\", \"{pwrd_hash}\", NOW(), NOW());"),
            // )).await?;

            // switch everything from varchars to text
            let query = "ALTER TABLE candidate MODIFY COLUMN name text; ALTER TABLE candidate MODIFY COLUMN surname text; ALTER TABLE candidate MODIFY COLUMN birth_surname text; ALTER TABLE candidate MODIFY COLUMN birthplace text; ALTER TABLE candidate MODIFY COLUMN address text; ALTER TABLE candidate MODIFY COLUMN letter_address text; ALTER TABLE candidate MODIFY COLUMN telephone text; ALTER TABLE candidate MODIFY COLUMN citizenship text; ALTER TABLE candidate MODIFY COLUMN email text; ALTER TABLE candidate MODIFY COLUMN sex text; ALTER TABLE candidate MODIFY COLUMN school_name text; ALTER TABLE candidate MODIFY COLUMN personal_identification_number text; ALTER TABLE candidate MODIFY COLUMN health_insurance text; ALTER TABLE candidate MODIFY COLUMN grades_json text; ALTER TABLE candidate MODIFY COLUMN first_school text; ALTER TABLE candidate MODIFY COLUMN second_school text; ALTER TABLE candidate MODIFY COLUMN test_language text; ALTER TABLE parent MODIFY COLUMN name text; ALTER TABLE parent MODIFY COLUMN surname text; ALTER TABLE parent MODIFY COLUMN telephone text; ALTER TABLE parent MODIFY COLUMN email text; ALTER TABLE application MODIFY COLUMN personal_id_number text; ALTER TABLE candidate MODIFY COLUMN birthdate text;".to_string();
            for a in query.split("; "){
                println!("executing {}", a);
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    a.to_string(),
                )).await?;
            }
        }

        Ok(SeaOrmPool { conn: db })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}
