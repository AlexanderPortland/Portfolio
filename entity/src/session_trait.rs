use async_trait::async_trait;
use sea_orm::prelude::Uuid;
use alohomora::bbox::BBox;
use portfolio_policies::FakePolicy;

#[async_trait]
pub trait UserSession {
    async fn expires_at(&self) -> BBox<chrono::NaiveDateTime, FakePolicy>;
    async fn id(&self) -> BBox<Uuid, FakePolicy>;
}