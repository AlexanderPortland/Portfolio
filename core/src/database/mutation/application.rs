use alohomora::policy::NoPolicy;
use alohomora::{bbox::BBox, policy::AnyPolicy};
use alohomora::pure::PrivacyPureRegion;
use ::entity::application;
use log::{info, warn};
use portfolio_policies::data::CandidateDataPolicy;
use rocket::form::name::Key;
use sea_orm::{DbConn, DbErr, Set, ActiveModelTrait, IntoActiveModel, DeleteResult, ModelTrait};
use portfolio_policies::{key::KeyPolicy, FakePolicy};

use crate::{Mutation, models::candidate::FieldOfStudy};

impl Mutation {
    pub async fn create_application(
        db: &DbConn,
        application_id: BBox<i32, CandidateDataPolicy>,
        candidate_id: BBox<i32, CandidateDataPolicy>,
        hashed_password: BBox<String, CandidateDataPolicy>,
        enc_personal_id_number: BBox<String, CandidateDataPolicy>,
        public_key: BBox<String, NoPolicy>,
        encrypted_private_key: BBox<String, KeyPolicy>,
    ) -> Result<application::Model, DbErr> {
        let field_of_study = application_id.clone().into_ppr(
            PrivacyPureRegion::new(|application_id| {
                FieldOfStudy::from(application_id).into()
            })
        );

        let insert = application::ActiveModel {
            id: Set(application_id.clone()),
            field_of_study: Set(field_of_study),
            personal_id_number: Set(enc_personal_id_number),
            password: Set(hashed_password),
            candidate_id: Set(candidate_id),
            public_key: Set(public_key),
            private_key: Set(encrypted_private_key),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), Default::default())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), Default::default())),
        }
            .insert(db)
            .await?;

        Ok(insert)
    }

    pub async fn delete_application(
        db: &DbConn,
        application: application::Model,
    ) -> Result<DeleteResult, DbErr> {
        let delete = application.delete(db).await?;
        Ok(delete)
    }

    pub async fn update_application_password_and_keys(
        db: &DbConn,
        application: application::Model,
        new_password_hash: BBox<String, CandidateDataPolicy>,
        public_key: BBox<String, NoPolicy>,
        private_key_encrypted: BBox<String, KeyPolicy>
    ) -> Result<application::Model, DbErr> {
        let mut application =  application.into_active_model();
        application.password = Set(new_password_hash);
        application.public_key = Set(public_key);
        application.private_key = Set(private_key_encrypted);

        let update = application.update(db).await?;

        Ok(update)
    }
}