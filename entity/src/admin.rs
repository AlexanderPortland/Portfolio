//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;
use alohomora::bbox::BBox;
use portfolio_policies::{KeyPolicy, FakePolicy};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "admin")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: BBox<i32, FakePolicy>,
    pub name: BBox<String, FakePolicy>,
    pub public_key: BBox<String, FakePolicy>,
    #[sea_orm(column_type = "Text")]
    pub private_key: BBox<String, KeyPolicy>,
    pub password: BBox<String, FakePolicy>,
    pub created_at: BBox<DateTime, FakePolicy>,
    pub updated_at: BBox<DateTime, FakePolicy>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::admin_session::Entity")]
    AdminSession,
}

impl Related<super::admin_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
