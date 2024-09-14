use alohomora::{bbox::BBox, policy::AnyPolicy};
use async_trait::async_trait;
use sea_orm::{prelude::Uuid, DbConn};
use portfolio_policies::{key::KeyPolicy, FakePolicy};

use crate::error::ServiceError;


#[async_trait]
pub trait AuthenticableTrait {
    type User;
    type Session;
    async fn login(db: &DbConn, user: BBox<i32, AnyPolicy>, password: BBox<String, AnyPolicy>, ip_addr: BBox<String, FakePolicy>) -> Result<(BBox<String, FakePolicy>, BBox<String, KeyPolicy>), ServiceError>;
    async fn auth(db: &DbConn, session_id: BBox<Uuid, FakePolicy>) -> Result<Self::User, ServiceError>;
    async fn logout(db: &DbConn, session: Self::Session) -> Result<(), ServiceError>;
    async fn new_session(db: &DbConn, user: &Self::User, password: BBox<String, AnyPolicy>, ip_addr: BBox<String, FakePolicy>) -> Result<BBox<String, FakePolicy>, ServiceError>;
    async fn delete_old_sessions(db: &DbConn, user: &Self::User, keep_n_recent: usize) -> Result<(), ServiceError>;
}