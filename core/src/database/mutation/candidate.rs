use crate::{Mutation, models::candidate_details::{EncryptedCandidateDetails}};

use alohomora::{bbox::BBox, policy::NoPolicy};
use ::entity::candidate;
use log::{info, warn};
use sea_orm::*;

impl Mutation {
    pub async fn create_candidate(
        db: &DbConn,
        enc_personal_id_number: BBox<String, NoPolicy>,
    ) -> Result<candidate::Model, DbErr> {
        let candidate = candidate::ActiveModel {
            personal_identification_number: Set(enc_personal_id_number),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(db)
            .await?;

        info!("CANDIDATE {} CREATED", candidate.clone().id.discard_box());
        Ok(candidate)
    }

    pub async fn delete_candidate(
        db: &DbConn,
        candidate: candidate::Model,
    ) -> Result<DeleteResult, DbErr> {
        let application = candidate.clone().id;
        let delete = candidate.delete(db).await?;

        warn!("CANDIDATE {} DELETED", application.discard_box());
        Ok(delete)
    }

    pub async fn update_candidate_opt_details(
        db: &DbConn,
        candidate: candidate::Model,
        enc_candidate: EncryptedCandidateDetails,
        encrypted_by_id: BBox<i32, NoPolicy>,
    ) -> Result<candidate::Model, sea_orm::DbErr> {
        let application = candidate.id.clone();
        let mut candidate: candidate::ActiveModel = candidate.into();

        candidate.name = Set(BBox::new(enc_candidate.name.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.surname = Set(BBox::new(enc_candidate.surname.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.birth_surname = Set(BBox::new(enc_candidate.birth_surname.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.birthplace = Set(BBox::new(enc_candidate.birthplace.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.birthdate = Set(BBox::new(enc_candidate.birthdate.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.address = Set(BBox::new(enc_candidate.address.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.letter_address = Set(BBox::new(enc_candidate.letter_address.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.telephone = Set(BBox::new(enc_candidate.telephone.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.citizenship = Set(BBox::new(enc_candidate.citizenship.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.email = Set(BBox::new(enc_candidate.email.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.sex = Set(BBox::new(enc_candidate.sex.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.school_name = Set(BBox::new(enc_candidate.school_name.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.health_insurance = Set(BBox::new(enc_candidate.health_insurance.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.grades_json = Set(BBox::new(enc_candidate.grades_json.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.first_school = Set(BBox::new(enc_candidate.first_school.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.second_school = Set(BBox::new(enc_candidate.second_school.discard_box().map(|e| e.into()), NoPolicy::new()));
        candidate.test_language = Set(BBox::new(enc_candidate.test_language.discard_box().map(|s| s), NoPolicy::new()));
        candidate.encrypted_by_id = Set(BBox::new(Some(encrypted_by_id.discard_box()), NoPolicy::new()));

        candidate.updated_at = Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new()));

        let update = candidate.update(db).await?;

        info!("CANDIDATE {} DETAILS UPDATED", application.discard_box());

        Ok(update)
    }

    pub async fn update_personal_id(
        db: &DbConn,
        candidate: candidate::Model,
        personal_id: &BBox<String, NoPolicy>,
    ) -> Result<candidate::Model, DbErr> {
        let mut candidate = candidate.into_active_model();
        candidate.personal_identification_number = Set(
            BBox::new(personal_id.clone().discard_box().to_string(), NoPolicy::new())
        );

        candidate
            .update(db)
            .await

    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;

    use crate::models::candidate_details::EncryptedApplicationDetails;
    use crate::models::candidate_details::tests::APPLICATION_DETAILS;
    use crate::utils::db::get_memory_sqlite_connection;
    use crate::{Mutation, Query};

    #[tokio::test]
    async fn test_create_candidate() {
        let db = get_memory_sqlite_connection().await;

        let candidate = Mutation::create_candidate(
            &db,
            BBox::new("".to_string(), NoPolicy::new()),
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
            BBox::new("".to_string(), NoPolicy::new()),
        )
        .await
        .unwrap();

        let encrypted_details: EncryptedApplicationDetails = EncryptedApplicationDetails::new(
            &APPLICATION_DETAILS.lock().unwrap().clone(),
            &vec!["age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_string()],
        ).await.unwrap();

        let candidate = Mutation::update_candidate_opt_details(&db, candidate, encrypted_details.candidate, BBox::new(1, NoPolicy::new())).await.unwrap();

        let candidate = Query::find_candidate_by_id(&db, candidate.id)
        .await
        .unwrap().unwrap();

        assert!(candidate.name.discard_box().is_some());
    }
}
