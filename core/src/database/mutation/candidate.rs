use crate::{Mutation, models::candidate_details::EncryptedCandidateDetails};

use alohomora::bbox::BBox;
use ::entity::candidate;
use log::{info, warn};
use sea_orm::*;
use portfolio_policies::{data::CandidateDataPolicy, FakePolicy};

impl Mutation {
    pub async fn create_candidate(
        db: &DbConn,
        enc_personal_id_number: BBox<String, CandidateDataPolicy>,
    ) -> Result<candidate::Model, DbErr> {
        let candidate = candidate::ActiveModel {
            personal_identification_number: Set(enc_personal_id_number),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), Default::default())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), Default::default())),
            ..Default::default()
        }
            .insert(db)
            .await?;

        Ok(candidate)
    }

    pub async fn delete_candidate(
        db: &DbConn,
        candidate: candidate::Model,
    ) -> Result<DeleteResult, DbErr> {
        let delete = candidate.delete(db).await?;
        Ok(delete)
    }

    pub async fn update_candidate_opt_details(
        db: &DbConn,
        candidate: candidate::Model,
        enc_candidate: EncryptedCandidateDetails,
        encrypted_by_id: BBox<i32, CandidateDataPolicy>,
    ) -> Result<candidate::Model, sea_orm::DbErr> {
        println!("specializinggg");
        let application = candidate.id.clone();
        println!("it all wne tokay :)");
        let mut candidate: candidate::ActiveModel = candidate.into();

        candidate.name = Set(enc_candidate.name.clone().map(|e| e.into()));
        candidate.surname = Set(enc_candidate.surname.clone().map(|e| e.into()));
        candidate.birth_surname = Set(enc_candidate.birth_surname.clone().map(|e| e.into()));
        candidate.birthplace = Set(enc_candidate.birthplace.clone().map(|e| e.into()));
        candidate.birthdate = Set(enc_candidate.birthdate.clone().map(|e| e.into()));
        candidate.address = Set(enc_candidate.address.clone().map(|e| e.into()));
        candidate.letter_address = Set(enc_candidate.letter_address.clone().map(|e| e.into()));
        candidate.telephone = Set(enc_candidate.telephone.clone().map(|e| e.into()));
        candidate.citizenship = Set(enc_candidate.citizenship.clone().map(|e| e.into()));
        candidate.email = Set(enc_candidate.email.clone().map(|e| e.into()));
        candidate.sex = Set(enc_candidate.sex.clone().map(|e| e.into()));
        candidate.school_name = Set(enc_candidate.school_name.clone().map(|e| e.into()));
        candidate.health_insurance = Set(enc_candidate.health_insurance.clone().map(|e| e.into()));
        candidate.grades_json = Set(enc_candidate.grades_json.clone().map(|e| e.into()));
        candidate.first_school = Set(enc_candidate.first_school.clone().map(|e| e.into()));
        candidate.second_school = Set(enc_candidate.second_school.clone().map(|e| e.into()));
        candidate.test_language = Set(enc_candidate.test_language.clone().map(|b| b.specialize_policy().unwrap()));
        candidate.encrypted_by_id = Set(Option::from(encrypted_by_id));
        candidate.updated_at = Set(BBox::new(chrono::offset::Local::now().naive_local(), Default::default()));

        let update = candidate.update(db).await?;
        Ok(update)
    }

    pub async fn update_personal_id(
        db: &DbConn,
        candidate: candidate::Model,
        personal_id: &BBox<String, CandidateDataPolicy>,
    ) -> Result<candidate::Model, DbErr> {
        let mut candidate = candidate.into_active_model();
        candidate.personal_identification_number = Set(personal_id.clone());
        candidate
            .update(db)
            .await

    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use portfolio_policies::data::CandidateDataPolicy;
    use portfolio_policies::FakePolicy;

    use crate::models::candidate_details::EncryptedApplicationDetails;
    use crate::models::candidate_details::tests::APPLICATION_DETAILS;
    use crate::utils::db::get_memory_sqlite_connection;
    use crate::{Mutation, Query};

    #[tokio::test]
    async fn test_create_candidate() {
        let db = get_memory_sqlite_connection().await;

        let candidate = Mutation::create_candidate(
            &db,
            BBox::new("".to_string(), CandidateDataPolicy::new(None)),
        )
        .await
        .unwrap();

        let candidate = Query::find_candidate_by_id(&db, candidate.id)
            .await
            .unwrap();
        assert!(candidate.is_some());
    }

    #[tokio::test]
    async fn test_add_candidate_details() {
        let db = get_memory_sqlite_connection().await;

        let candidate = Mutation::create_candidate(
            &db,
            BBox::new("".to_string(), CandidateDataPolicy::new(None)),
        )
        .await
        .unwrap();

        let encrypted_details: EncryptedApplicationDetails = EncryptedApplicationDetails::new(
            &APPLICATION_DETAILS.lock().unwrap().clone(),
            &vec![BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_string(), NoPolicy::new())],
        ).await.unwrap();

        let candidate = Mutation::update_candidate_opt_details(&db, candidate, encrypted_details.candidate, BBox::new(1, CandidateDataPolicy::new(None))).await.unwrap();

        let candidate = Query::find_candidate_by_id(&db, candidate.id)
        .await
        .unwrap().unwrap();

        assert!(candidate.name.is_some());
    }
}
