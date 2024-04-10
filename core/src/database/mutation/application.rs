use alohomora::{bbox::BBox, policy::NoPolicy};
use ::entity::application;
use log::{info, warn};
use sea_orm::{DbConn, DbErr, Set, ActiveModelTrait, IntoActiveModel, DeleteResult, ModelTrait};

use crate::{Mutation, models::candidate::FieldOfStudy};

impl Mutation {
    pub async fn create_application(
        db: &DbConn,
        application_id: BBox<i32, NoPolicy>,
        candidate_id: BBox<i32, NoPolicy>,
        hashed_password: BBox<String, NoPolicy>,
        enc_personal_id_number: BBox<String, NoPolicy>,
        pubkey: BBox<String, NoPolicy>,
        encrypted_priv_key: BBox<String, NoPolicy>,
    ) -> Result<application::Model, DbErr> {
        let field_of_study = FieldOfStudy::from(application_id.clone().discard_box());
        let insert = application::ActiveModel {
            id: Set(application_id.clone()),
            field_of_study: Set(BBox::new(field_of_study.into(), NoPolicy::new())),
            personal_id_number: Set(enc_personal_id_number),
            password: Set(hashed_password),
            candidate_id: Set(candidate_id),
            public_key: Set(pubkey),
            private_key: Set(encrypted_priv_key),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
        }
            .insert(db)
            .await?;

        info!("APPLICATION {} CREATED", application_id.discard_box());
        Ok(insert)
    }

    pub async fn delete_application(
        db: &DbConn,
        application: application::Model,
    ) -> Result<DeleteResult, DbErr> {
        let application_id = application.id.clone();
        let delete = application.delete(db).await?;

        warn!("APPLICATION {} DELETED", application_id.discard_box());
        Ok(delete)
    }

    pub async fn update_application_password_and_keys(
        db: &DbConn,
        application: application::Model,
        new_password_hash: BBox<String, NoPolicy>,
        pub_key: BBox<String, NoPolicy>,
        priv_key_enc: BBox<String, NoPolicy>
    ) -> Result<application::Model, DbErr> {
        let application_id = application.id.clone();
        let mut application =  application.into_active_model();
        application.password = Set(new_password_hash);
        application.public_key = Set(pub_key);
        application.private_key = Set(priv_key_enc);

        let update = application.update(db).await?;

        warn!("CANDIDATE {} PASSWORD CHANGED", application_id.discard_box());
        Ok(update)
    }
}