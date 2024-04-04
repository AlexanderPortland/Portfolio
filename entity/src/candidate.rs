//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use alohomora::policy::NoPolicy;
use sea_orm::entity::prelude::*;
use alohomora::bbox::BBox;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "candidate")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: BBox<i32, NoPolicy>,
    pub name: BBox<Option<String>, NoPolicy>,
    pub surname: BBox<Option<String>, NoPolicy>,
    pub birth_surname: BBox<Option<String>, NoPolicy>,
    pub birthplace: BBox<Option<String>, NoPolicy>,
    pub birthdate: BBox<Option<String>, NoPolicy>,
    pub address: BBox<Option<String>, NoPolicy>,
    pub letter_address: BBox<Option<String>, NoPolicy>,
    pub telephone: BBox<Option<String>, NoPolicy>,
    pub citizenship: BBox<Option<String>, NoPolicy>,
    pub email: BBox<Option<String>, NoPolicy>,
    pub sex: BBox<Option<String>, NoPolicy>,
    pub personal_identification_number: BBox<String, NoPolicy>,
    pub school_name: BBox<Option<String>, NoPolicy>,
    pub health_insurance: BBox<Option<String>, NoPolicy>,
    pub grades_json: BBox<Option<String>, NoPolicy>,
    pub first_school: BBox<Option<String>, NoPolicy>,
    pub second_school: BBox<Option<String>, NoPolicy>,
    pub test_language: BBox<Option<String>, NoPolicy>,
    pub encrypted_by_id: BBox<Option<i32>, NoPolicy>,
    pub created_at: BBox<DateTime, NoPolicy>,
    pub updated_at: BBox<DateTime, NoPolicy>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::application::Entity")]
    Application,
    #[sea_orm(has_many = "super::parent::Entity")]
    Parent,
}

impl Related<super::application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Application.def()
    }
}

impl Related<super::parent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
