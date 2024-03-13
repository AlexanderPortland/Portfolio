use async_trait::async_trait;
use portfolio_core::sea_orm::{self};
#[cfg(not(test))]
use sea_orm::ConnectOptions;
use sea_orm_rocket::{rocket::figment::Figment, Database};
#[cfg(not(test))]
use std::time::Duration;
use entity::{admin_session, application};
use sea_orm::DbConn;

#[derive(Database, Debug)]
#[database("sea_orm")]
pub struct Db(SeaOrmPool);

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
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

        use sea_orm::{Schema, Database, Statement};
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

        if true {
            use entity::{admin, candidate, parent, session};
            

            let schema = Schema::new(DbBackend::MySql);
            let stmt: TableCreateStatement = schema.create_table_from_entity(candidate::Entity);
            let stmt2: TableCreateStatement = schema.create_table_from_entity(application::Entity);
            let stmt3: TableCreateStatement = schema.create_table_from_entity(session::Entity);
            let stmt4: TableCreateStatement = schema.create_table_from_entity(admin::Entity);
            let stmt5: TableCreateStatement = schema.create_table_from_entity(admin_session::Entity);
            let stmt6: TableCreateStatement = schema.create_table_from_entity(parent::Entity);
            // need to enter correct db
            //let _ = db.execute(Statement::from_string(db.get_database_backend(), "CREATE DATABASE portfolio;".to_string())).await;
            //let _ = db.execute(Statement::from_string(db.get_database_backend(), "USE portfolio;".to_string())).await;
            println!("helskdjklgajklsd");
            println!("stmt is {:?}", stmt);
            let b = db.get_database_backend().build(&stmt);
            println!("b is {:?}", b);
            let r = db.execute(b).await;
            println!("res is {:?}", r);
            r.unwrap();
            db.execute(db.get_database_backend().build(&stmt2)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt3)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt4)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt5)).await.unwrap();
            db.execute(db.get_database_backend().build(&stmt6)).await.unwrap();
        }
        // have to convert form DbConn to databaseconnection
        // what hte hells hte difference???
        // just type aliases lmao
        Ok(SeaOrmPool { conn: db })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}
