//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;
use alohomora::{bbox::BBox, policy::NoPolicy};
use portfolio_policies::{data::CandidateDataPolicy, key::KeyPolicy, FakePolicy};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "application")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: BBox<i32, CandidateDataPolicy>,
    pub candidate_id: BBox<i32, CandidateDataPolicy>,
    pub field_of_study: BBox<String, CandidateDataPolicy>,
    pub password: BBox<String, CandidateDataPolicy>,
    pub public_key: BBox<String, NoPolicy>,
    pub private_key: BBox<String, KeyPolicy>,
    pub personal_id_number: BBox<String, CandidateDataPolicy>,
    pub created_at: BBox<DateTime, CandidateDataPolicy>,
    pub updated_at: BBox<DateTime, CandidateDataPolicy>,
}

// make another BBoxModel struct and way to transform between for now 
// until we can implement it ourself
// no policy everywhere
// start from the database and then see where that gets errors and work out from there
// transformation functions in the Query::___ fn so only the body of that will change
// and then use .discard_box to get rid of boxes and still return a Json

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::candidate::Entity",
        from = "Column::CandidateId",
        to = "super::candidate::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Candidate,
    #[sea_orm(has_many = "super::session::Entity")]
    Session,
}

impl Related<super::candidate::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Candidate.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
