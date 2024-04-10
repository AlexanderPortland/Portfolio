use alohomora::{bbox::BBox, policy::NoPolicy};
use sea_orm::*;

use ::entity::{candidate, candidate::Entity as Candidate};

use crate::Query;

pub const PAGE_SIZE: u64 = 20;

#[derive(FromQueryResult)]
pub struct IdPersonalIdNumberJoin {
    pub id: BBox<i32, NoPolicy>,
    pub personal_id_number: BBox<String, NoPolicy>,
}

#[derive(FromQueryResult)]
pub struct ApplicationId {
    application: BBox<i32, NoPolicy>,
}

impl ApplicationId {
    pub fn to_i32(&self) -> BBox<i32, NoPolicy> {
        self.application
    }
}

#[derive(FromQueryResult, Clone)]
pub struct CandidateResult {
    pub application: BBox<i32, NoPolicy>,
    pub name: BBox<Option<String>, NoPolicy>,
    pub surname: BBox<Option<String>, NoPolicy>,
    pub email: BBox<Option<String>, NoPolicy>,
    pub telephone: BBox<Option<String>, NoPolicy>,
    pub study: BBox<Option<String>, NoPolicy>,
    pub citizenship: BBox<Option<String>, NoPolicy>,
}

impl Query {
    pub async fn find_candidate_by_id(
        db: &DbConn,
        id: BBox<i32, NoPolicy>,
    ) -> Result<Option<candidate::Model>, DbErr> {
        Candidate::find_by_id(id)
            .one(db)
            .await
    }

    pub async fn list_candidates_full(
        db: &DbConn
    ) -> Result<Vec<candidate::Model>, DbErr> {
        Candidate::find()
            .order_by(candidate::Column::Id, Order::Asc)
            .all(db)
            .await
    }

    pub async fn list_all_candidate_ids(
        db: &DbConn,
    ) -> Result<Vec<ApplicationId>, DbErr> {
        Candidate::find()
            .order_by(candidate::Column::Id, Order::Asc)
            .column(candidate::Column::Id)
            .into_model::<ApplicationId>()
            .all(db)
            .await
    }

    pub async fn find_candidate_by_personal_id(
        db: &DbConn,
        personal_id: BBox<&str, NoPolicy>,
    ) -> Result<Option<candidate::Model>, DbErr> {
        Candidate::find()
            .filter(candidate::Column::PersonalIdentificationNumber.eq(personal_id))
            .one(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use sea_orm::{ActiveModelTrait, Set};

    use entity::candidate;

    use crate::Query;
    use crate::utils::db::get_memory_sqlite_connection;

    #[tokio::test]
    async fn test_find_candidate_by_id() {
        let db = get_memory_sqlite_connection().await;
        let candidate = candidate::ActiveModel {
            id: Set(BBox::new(103158, NoPolicy::new())),
            personal_identification_number: Set(BBox::new("test".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let candidate = Query::find_candidate_by_id(&db, candidate.id)
            .await
            .unwrap();
        assert!(candidate.is_some());
    }
}
