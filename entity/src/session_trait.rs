use alohomora::policy::NoPolicy;
use async_trait::async_trait;
use sea_orm::prelude::Uuid;
use alohomora::bbox::BBox;

#[async_trait]
pub trait UserSession {
    async fn expires_at(&self) -> BBox<chrono::NaiveDateTime, NoPolicy>;
    async fn id(&self) -> BBox<Uuid, NoPolicy>;
}