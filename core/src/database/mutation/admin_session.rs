use alohomora::bbox::BBox;
use chrono::{Utc, Duration};
use entity::admin_session;
use sea_orm::{DbConn, prelude::Uuid, DbErr, Set, ActiveModelTrait};
use portfolio_policies::FakePolicy;

use crate::Mutation;

impl Mutation {
    pub async fn insert_admin_session(
        db: &DbConn,
        admin_id: BBox<i32, FakePolicy>,
        random_uuid: BBox<Uuid, FakePolicy>,
        ip_addr: BBox<String, FakePolicy>,
    ) -> Result<admin_session::Model, DbErr> {
        admin_session::ActiveModel {
            id: Set(random_uuid),
            admin_id: Set(admin_id),
            ip_address: Set(ip_addr),
            created_at: Set(BBox::new(Utc::now().naive_local(), Default::default())),
            expires_at: Set(BBox::new(Utc::now()
                .naive_local()
                .checked_add_signed(Duration::days(1))
                .unwrap(), Default::default())),
            updated_at: Set(BBox::new(Utc::now().naive_local(), Default::default()))
        }
        .insert(db)
        .await
    }

    /* pub async fn update_session_expiration(db: &DbConn, 
        session: session::Model, 
        expires_at: NaiveDateTime,
    ) -> Result<session::Model, DbErr> {
        let mut session = session.into_active_model();

        session.expires_at = Set(expires_at);
        session.updated_at = Set(Utc::now().naive_local());
        
        session.update(db).await
    }

    pub async fn delete_admin_session(db: &DbConn, session: ad) -> Result<DeleteResult, DbErr> {
        session::ActiveModel {
            id: Set(session_id),
            ..Default::default()
        }
            .delete(db)
            .await
    } */
}