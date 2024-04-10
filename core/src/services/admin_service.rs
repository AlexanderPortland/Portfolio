use async_trait::async_trait;
use entity::{admin, admin_session, session};
use sea_orm::{prelude::Uuid, DbConn, IntoActiveModel};
use alohomora::{bbox::BBox, policy::NoPolicy};

use crate::{crypto, error::ServiceError, Query, Mutation, models::auth::AuthenticableTrait};

use super::session_service::SessionService;

pub struct AdminService;

impl AdminService {
    async fn decrypt_private_key(
        db: &DbConn,
        admin_id: BBox<i32, NoPolicy>,
        password: BBox<String, NoPolicy>,
    ) -> Result<BBox<String, NoPolicy>, ServiceError> {
        let admin = Query::find_admin_by_id(db, admin_id).await?.ok_or(ServiceError::InvalidCredentials)?;
        let private_key_encrypted = admin.private_key;

        // ALO: thinking pcr here
        let private_key = crypto::decrypt_password(private_key_encrypted.discard_box(), password.discard_box()).await?;
        let private_key = BBox::new(private_key, NoPolicy::new());

        Ok(private_key)
    }
}

#[async_trait]
impl AuthenticableTrait for AdminService {
    type User = admin::Model;
    type Session = admin_session::Model;

    async fn login(
        db: &DbConn,
        admin_id: BBox<i32, NoPolicy>,
        password: BBox<String, NoPolicy>,
        ip_addr: BBox<String, NoPolicy>,
    ) -> Result<(BBox<String, NoPolicy>, BBox<String, NoPolicy>), ServiceError> {
        let admin = Query::find_admin_by_id(db, admin_id).await?.ok_or(ServiceError::InvalidCredentials)?;
        
        let session_id = Self::new_session(db,
            &admin,
            password.clone(),
            ip_addr
        )
            .await?;
        
        // ALO: maybe sandbox here?
        //let private_key = Self::decrypt_private_key(db, admin.id, password).await?;
        let private_key = BBox::new("13".to_string(), NoPolicy::new());

        Ok((session_id, private_key))
    }

    async fn auth(db: &DbConn, session_uuid: BBox<Uuid, NoPolicy>) -> Result<admin::Model, ServiceError> {
        let session = Query::find_admin_session_by_uuid(db, session_uuid)
            .await?
            .ok_or(ServiceError::Unauthorized)?;

        if !SessionService::is_valid(&session).await? {
            Mutation::delete_session(db, session.into_active_model()).await?;
            return Err(ServiceError::ExpiredSession);
        }

        let admin = Query::find_admin_by_id(db, session.admin_id)
            .await?
            .ok_or(ServiceError::CandidateNotFound)?;

        Ok(admin)
    }

    async fn logout(db: &DbConn, session: admin_session::Model) -> Result<(), ServiceError> {
        Mutation::delete_session(db, session.into_active_model()).await?;
        Ok(())
    }

    async fn new_session(
        db: &DbConn,
        admin: &admin::Model,
        password: BBox<String, NoPolicy>,
        ip_addr: BBox<String, NoPolicy>,
    ) -> Result<BBox<String, NoPolicy>, ServiceError> {
        if !crypto::verify_password(password.clone().discard_box(), admin.password.clone().discard_box()).await? {
            println!("crypto couldn't verify that password");
            return Err(ServiceError::InvalidCredentials);
        }
        // user is authenticated, generate a new session
        let random_uuid: Uuid = Uuid::new_v4();

        let session = Mutation::insert_admin_session(db, admin.id, BBox::new(random_uuid, NoPolicy::new()), ip_addr).await?;

        Self::delete_old_sessions(db, &admin, 1).await?;
        let s = session.id.discard_box().to_string();
        Ok(BBox::new(s, NoPolicy::new()))
    }
    async fn delete_old_sessions(
        db: &DbConn,
        admin: &admin::Model,
        keep_n_recent: usize,
    ) -> Result<(), ServiceError> {
        let sessions = Query::find_related_admin_sessions(db, admin)
            .await?
            .iter()
            .map(|s| s.clone().into_active_model())
            .collect();

        SessionService::delete_sessions(db, sessions, keep_n_recent).await?;
        Ok(())
    }

}

//#[cfg(test)]
pub mod admin_tests {
    use chrono::{Local, Utc};
    use entity::admin;
    use sea_orm::{Set, ActiveModelTrait};
    
    use crate::{utils::db::get_memory_sqlite_connection, error::ServiceError};
    
    use super::*;
    
    pub async fn create_admin(db: &DbConn) -> admin::Model {    
        let password = "admin".to_string();
        let (pubkey, priv_key) = crypto::create_identity();

        // TODO: is this the right parameter order for password encryption
        let enc_priv_key = crypto::encrypt_password(priv_key.discard_box(), password).await.unwrap();
        let enc_priv_key = BBox::new(enc_priv_key, NoPolicy::new());

        let admin = admin::ActiveModel {
            name: Set(BBox::new("admin".to_string(), NoPolicy::new())),
            public_key: Set(pubkey),
            private_key: Set(enc_priv_key),
            // should be password hash
            password: Set(BBox::new("admin".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(Utc::now().naive_utc(), NoPolicy::new())),
            updated_at: Set(BBox::new(Utc::now().naive_utc(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(db)
            .await
            .unwrap();
    
        admin
    }

    #[tokio::test]
    async fn test_admin_login() -> Result<(), ServiceError> {
        let db = get_memory_sqlite_connection().await;
        let admin = admin::ActiveModel {
            id: Set(BBox::new(1, NoPolicy::new())),
            name: Set(BBox::new("Admin".to_owned(), NoPolicy::new())),
            public_key: Set(BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_owned(), NoPolicy::new())),
            // AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS
            private_key: Set(BBox::new("5KCEGk0ueWVGnu5Xo3rmpLoilcVZ2ZWmwIcdZEJ8rrBNW7jwzZU/XTcTXtk/xyy/zjF8s+YnuVpOklQvX3EC/Sn+ZwyPY3jokM2RNwnZZlnqdehOEV1SMm/Y".to_owned(), NoPolicy::new())),
            // test
            password: Set(BBox::new("$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ".to_owned(), NoPolicy::new())),
            created_at: Set(BBox::new(Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(&db)
            .await?;

        let (session_id, _private_key) = AdminService::login(&db, admin.id, 
            BBox::new("test".to_owned(), NoPolicy::new()), 
            BBox::new("127.0.0.1".to_owned(), NoPolicy::new())).await?;

        let logged_admin = AdminService::auth(&db, BBox::new(session_id.discard_box().parse().unwrap(), NoPolicy::new())).await?;

        assert_eq!(logged_admin.id.discard_box(), 1);
        assert_eq!(logged_admin.name.discard_box(), "Admin");
        

        Ok(())

    }
}