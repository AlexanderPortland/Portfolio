use alohomora::bbox::BBox;
use sea_orm::*;

use ::entity::{candidate, candidate::Entity as Candidate};
use portfolio_policies::{data::CandidateDataPolicy, FakePolicy};

use crate::Query;

pub const PAGE_SIZE: u64 = 20;

#[derive(FromQueryResult)]
pub struct IdPersonalIdNumberJoin {
    pub id: BBox<i32, FakePolicy>,
    pub personal_id_number: BBox<String, FakePolicy>,
}

#[derive(FromQueryResult)]
pub struct ApplicationId {
    application: BBox<i32, FakePolicy>,
}

impl ApplicationId {
    pub fn to_i32(&self) -> BBox<i32, FakePolicy> {
        self.application.clone()
    }
}

#[derive(FromQueryResult, Clone)]
pub struct CandidateResult {
    pub application: BBox<i32, FakePolicy>,
    pub name: BBox<Option<String>, FakePolicy>,
    pub surname: BBox<Option<String>, FakePolicy>,
    pub email: BBox<Option<String>, FakePolicy>,
    pub telephone: BBox<Option<String>, FakePolicy>,
    pub study: BBox<Option<String>, FakePolicy>,
    pub citizenship: BBox<Option<String>, FakePolicy>,
}

impl Query {
    pub async fn find_candidate_by_id(
        db: &DbConn,
        id: BBox<i32, CandidateDataPolicy>,
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
        personal_id: BBox<&str, FakePolicy>,
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
    use portfolio_policies::data::CandidateDataPolicy;
    use sea_orm::{ActiveModelTrait, Set};

    use entity::candidate;
    use portfolio_policies::FakePolicy;

    use crate::Query;
    use crate::utils::db::get_memory_sqlite_connection;

    const CANDIDATE_ID: i32 = 103158;
    #[tokio::test]
    async fn test_find_candidate_by_id() {
        let db = get_memory_sqlite_connection().await;
        let candidate = candidate::ActiveModel {
            id: Set(BBox::new(CANDIDATE_ID, CandidateDataPolicy::new(Some(CANDIDATE_ID)))),
            personal_identification_number: Set(BBox::new("test".to_string(), CandidateDataPolicy::new(Some(CANDIDATE_ID)))),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), FakePolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), FakePolicy::new())),
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
