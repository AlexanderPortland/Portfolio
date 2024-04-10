use std::net::IpAddr;

use alohomora::{bbox::BBox, policy::NoPolicy};
use chrono::{Utc, Duration};
use entity::{admin, admin_session};
use sea_orm::{DbConn, prelude::Uuid, DbErr, Set, ActiveModelTrait};

use crate::Mutation;

impl Mutation {
    pub async fn insert_admin_session(
        db: &DbConn,
        admin_id: BBox<i32, NoPolicy>,
        random_uuid: BBox<Uuid, NoPolicy>,
        ip_addr: BBox<String, NoPolicy>,
    ) -> Result<admin_session::Model, DbErr> {
        admin_session::ActiveModel {
            id: Set(random_uuid),
            admin_id: Set(admin_id),
            ip_address: Set(ip_addr),
            created_at: Set(BBox::new(Utc::now().naive_local(), NoPolicy::new())),
            expires_at: Set(BBox::new(Utc::now()
                .naive_local()
                .checked_add_signed(Duration::days(1))
                .unwrap(), NoPolicy::new())),
            updated_at: Set(BBox::new(Utc::now().naive_local(), NoPolicy::new()))
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