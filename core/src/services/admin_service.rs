use async_trait::async_trait;
use entity::{admin, admin_session};
use sea_orm::{prelude::Uuid, DbConn, IntoActiveModel};
use alohomora::bbox::BBox;
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use portfolio_policies::FakePolicy;

use crate::{crypto, error::ServiceError, Query, Mutation, models::auth::AuthenticableTrait};
use crate::crypto_helpers::{my_decrypt_password, my_verify_password};

use super::session_service::SessionService;

pub struct AdminService;

impl AdminService {
    async fn decrypt_private_key(
        db: &DbConn,
        admin_id: BBox<i32, FakePolicy>,
        password: BBox<String, FakePolicy>,
    ) -> Result<BBox<String, FakePolicy>, ServiceError> {
        let admin = Query::find_admin_by_id(db, admin_id).await?.ok_or(ServiceError::InvalidCredentials)?;
        let private_key_encrypted = admin.private_key;
        my_decrypt_password(private_key_encrypted, password).await
    }
}

#[async_trait]
impl AuthenticableTrait for AdminService {
    type User = admin::Model;
    type Session = admin_session::Model;

    async fn login(
        db: &DbConn,
        admin_id: BBox<i32, FakePolicy>,
        password: BBox<String, FakePolicy>,
        ip_addr: BBox<String, FakePolicy>,
    ) -> Result<(BBox<String, FakePolicy>, BBox<String, FakePolicy>), ServiceError> {
        let admin = Query::find_admin_by_id(db, admin_id).await?.ok_or(ServiceError::InvalidCredentials)?;
        let session_id = Self::new_session(db,
            &admin,
            password.clone(),
            ip_addr
        )
            .await?;

        let private_key = Self::decrypt_private_key(db, admin.id, password).await?;

        Ok((session_id, private_key))
    }

    async fn auth(db: &DbConn, session_uuid: BBox<Uuid, FakePolicy>) -> Result<admin::Model, ServiceError> {
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
        password: BBox<String, FakePolicy>,
        ip_addr: BBox<String, FakePolicy>,
    ) -> Result<BBox<String, FakePolicy>, ServiceError> {
        if !my_verify_password(password.clone(), admin.password.clone()).await? {
            println!("crypto couldn't verify that password");
            return Err(ServiceError::InvalidCredentials);
        }

        // user is authenticated, generate a new session
        let random_uuid = BBox::new(Uuid::new_v4(), FakePolicy::new());

        let session = Mutation::insert_admin_session(db, admin.id.clone(), random_uuid, ip_addr).await?;

        Self::delete_old_sessions(db, &admin, 1).await?;

        Ok(session.id.into_ppr(PrivacyPureRegion::new(|id: Uuid| id.to_string())))
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

#[cfg(test)]
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
        let enc_priv_key = crypto::encrypt_password(priv_key, password).await.unwrap();
        let enc_priv_key = BBox::new(enc_priv_key, FakePolicy::new());

        let admin = admin::ActiveModel {
            name: Set(BBox::new("admin".to_string(), FakePolicy::new())),
            public_key: Set(pubkey),
            private_key: Set(enc_priv_key),
            // should be password hash
            password: Set(BBox::new("admin".to_string(), FakePolicy::new())),
            created_at: Set(BBox::new(Utc::now().naive_utc(), FakePolicy::new())),
            updated_at: Set(BBox::new(Utc::now().naive_utc(), FakePolicy::new())),
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
            id: Set(BBox::new(1, FakePolicy::new())),
            name: Set(BBox::new("Admin".to_owned(), FakePolicy::new())),
            public_key: Set(BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_owned(), FakePolicy::new())),
            // AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS
            private_key: Set(BBox::new("5KCEGk0ueWVGnu5Xo3rmpLoilcVZ2ZWmwIcdZEJ8rrBNW7jwzZU/XTcTXtk/xyy/zjF8s+YnuVpOklQvX3EC/Sn+ZwyPY3jokM2RNwnZZlnqdehOEV1SMm/Y".to_owned(), FakePolicy::new())),
            // test
            password: Set(BBox::new("$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ".to_owned(), FakePolicy::new())),
            created_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
            updated_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
            ..Default::default()
        }
            .insert(&db)
            .await?;

        let (session_id, _private_key) = AdminService::login(&db, admin.id, 
            BBox::new("test".to_owned(), FakePolicy::new()),
            BBox::new("127.0.0.1".to_owned(), FakePolicy::new())).await?;

        let logged_admin = AdminService::auth(&db, BBox::new(session_id.discard_box().parse().unwrap(), FakePolicy::new())).await?;

        assert_eq!(logged_admin.id.discard_box(), 1);
        assert_eq!(logged_admin.name.discard_box(), "Admin");
        

        Ok(())

    }
}