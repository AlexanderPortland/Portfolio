use alohomora::{bbox::BBox, policy::NoPolicy};
use async_trait::async_trait;
use sea_orm::{prelude::Uuid, DbConn};

use crate::error::ServiceError;


#[async_trait]
pub trait AuthenticableTrait {
    type User;
    type Session;
    async fn login(db: &DbConn, user: BBox<i32, NoPolicy>, password: BBox<String, NoPolicy>, ip_addr: BBox<String, NoPolicy>) -> Result<(BBox<String, NoPolicy>, BBox<String, NoPolicy>), ServiceError>;
    async fn auth(db: &DbConn, session_id: BBox<Uuid, NoPolicy>) -> Result<Self::User, ServiceError>;
    async fn logout(db: &DbConn, session: Self::Session) -> Result<(), ServiceError>;
    async fn new_session(db: &DbConn, user: &Self::User, ip_addr: BBox<String, NoPolicy>, password: BBox<String, NoPolicy>) -> Result<BBox<String, NoPolicy>, ServiceError>;
    async fn delete_old_sessions(db: &DbConn, user: &Self::User, keep_n_recent: usize) -> Result<(), ServiceError>;
}