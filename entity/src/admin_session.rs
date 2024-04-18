//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

use crate::session_trait::UserSession;
use alohomora::bbox::BBox;
use portfolio_policies::FakePolicy;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: BBox<Uuid, FakePolicy>,
    pub admin_id: BBox<i32, FakePolicy>,
    pub ip_address: BBox<String, FakePolicy>,
    pub created_at: BBox<DateTime, FakePolicy>,
    pub expires_at: BBox<DateTime, FakePolicy>,
    pub updated_at: BBox<DateTime, FakePolicy>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin::Entity",
        from = "Column::AdminId",
        to = "super::admin::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Admin,
}

impl Related<super::admin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Admin.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait::async_trait]
impl UserSession for Model {
    async fn id(&self) -> BBox<Uuid, FakePolicy> {
        self.id.clone()
    }
    async fn expires_at(&self) -> BBox<chrono::NaiveDateTime, FakePolicy> {
        self.expires_at.clone()
    }
}
