use std::default;

use alohomora::context::{Context, ContextData};
use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime};
use entity::{candidate, parent, application, session};
use portfolio_policies::context::ContextDataType;
use sea_orm::{DbConn, prelude::Uuid, IntoActiveModel};
use alohomora::bbox::BBox;
use alohomora::policy::AnyPolicy;
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use entity::session_trait::UserSession;
use portfolio_policies::FakePolicy;

use crate::{error::ServiceError, Query, utils::db::get_recipients, models::candidate_details::EncryptedApplicationDetails, models::{candidate::{ApplicationDetails, CreateCandidateResponse}, candidate_details::{EncryptedString, EncryptedCandidateDetails}, auth::AuthenticableTrait, application::ApplicationResponse}, Mutation, crypto};
use crate::crypto_helpers::{my_decrypt_password, my_encrypt_password, my_hash_password, my_verify_password};

use super::{parent_service::ParentService, candidate_service::CandidateService, session_service::SessionService, portfolio_service::{PortfolioService, SubmissionProgress}};

const FIELD_OF_STUDY_PREFIXES: [&str; 3] = ["101", "102", "103"];

pub struct ApplicationService;

impl ApplicationService {
    /// Creates a new candidate with:
    /// Encrypted personal identification number
    /// Hashed password
    /// Encrypted private key
    /// Public key
    pub async fn create<D: ContextData + Clone>(
        context: Context<D>,
        admin_private_key: &BBox<String, FakePolicy>,
        db: &DbConn,
        application_id: BBox<i32, FakePolicy>,
        plain_text_password: &BBox<String, FakePolicy>,
        personal_id_number: BBox<String, FakePolicy>,
    ) -> Result<(application::Model, Vec<application::Model>, BBox<String, FakePolicy>), ServiceError> {
        // Check if application id starts with 101, 102 or 103
        application_id.clone().into_ppr(PrivacyPureRegion::new(|id| {
            if !Self::is_application_id_valid(id) {
                return Err(ServiceError::InvalidApplicationId);
            }
            Ok(())
        })).transpose()?;

        // Check if user with that application id already exists
        if Query::find_application_by_id(db, application_id.clone())
            .await?
            .is_some()
        {
            println!("user exists");
            return Err(ServiceError::UserAlreadyExists);
        }
        
        let hashed_password = my_hash_password(plain_text_password.to_owned()).await?;
        let (pubkey, priv_key_plain_text) = crypto::create_identity();
        let pubkey = BBox::new(pubkey, FakePolicy::new());
        let encrypted_priv_key = my_encrypt_password(
            priv_key_plain_text,
            plain_text_password.to_owned(),
        ).await?;

        let (candidate, enc_personal_id_number) = Self::find_or_create_candidate_with_personal_id(
            context.clone(),
            application_id.clone(),
            admin_private_key,
            db,
            &personal_id_number,
            &pubkey,
        ).await?;

        let application = Mutation::create_application(
            db,
            application_id,
            candidate.id.clone(),
            hashed_password,
            enc_personal_id_number,
            pubkey,
            encrypted_priv_key,
        ).await?;

        let applications = Query::find_applications_by_candidate_id(db, candidate.id).await?;
        if applications.len() >= 3 {
            for application in applications {
                ApplicationService::delete(context.clone(), db, application).await?;
            }
            return Err(ServiceError::InternalServerError);
        }
        Ok(
            (
                application,
                applications,
                personal_id_number,
            )
        )
    }

    async fn find_or_create_candidate_with_personal_id<D: ContextData + Clone>(
        context: Context<D>,
        application_id: BBox<i32, FakePolicy>,
        admin_private_key: &BBox<String, FakePolicy>,
        db: &DbConn,
        personal_id_number: &BBox<String, FakePolicy>,
        pubkey: &BBox<String, FakePolicy>,
        // enc_personal_id_number: &EncryptedString,
    ) -> Result<(candidate::Model, BBox<String, FakePolicy>), ServiceError> {
        let candidates = Query::list_candidates_full(db).await?;
        let ids_decrypted = futures::future::join_all(
            candidates.iter().map(|c| async {
                let es = match EncryptedString::from(c.personal_identification_number.clone())
                    .decrypt(admin_private_key)
                    .await{
                        Ok(bbox) => Ok(bbox.specialize_policy().unwrap()),
                        Err(e) => Err(e),
                }.unwrap_or_default();
                
                (c.id.clone(), es)
            })
        ).await;

        let found_ids: Vec<&(BBox<i32, FakePolicy>, BBox<String, FakePolicy>)> = ids_decrypted
            .iter()
            .filter(|(_, id)| id == personal_id_number)
            .collect();

        if let Some((candidate_id, _)) = found_ids.first() {
            Ok(
                Self::find_linkable_candidate(db, 
                    application_id,
                    candidate_id.clone(),
                    pubkey,
                    personal_id_number.to_owned()
                ).await?
            )
        } else {
            let recipients = get_recipients(db, pubkey.clone()).await?;
            let enc_personal_id_number: BBox<String, FakePolicy> = EncryptedString::new(
                personal_id_number.clone(),
                &recipients,
            ).await?.into();

            Ok(
                (
                    CandidateService::create(context, db, enc_personal_id_number.clone()).await?,
                    enc_personal_id_number,
                )
            )
        }
    }

    async fn find_linkable_candidate(
        db: &DbConn,
        new_application_id: BBox<i32, FakePolicy>,
        candidate_id: BBox<i32, FakePolicy>,
        pubkey: &BBox<String, FakePolicy>,
        personal_id_number: BBox<String, FakePolicy>,
    ) -> Result<(candidate::Model, BBox<String, FakePolicy>), ServiceError> {
        let candidate = Query::find_candidate_by_id(db, candidate_id)
            .await?
            .ok_or(ServiceError::CandidateNotFound)?;
                
        let linked_applications = Query::find_applications_by_candidate_id(db, candidate.id.clone()).await?;

        if linked_applications.len() > 1 {
            return Err(ServiceError::TooManyApplications);
        }

        let linked_application = linked_applications.first().ok_or(ServiceError::CandidateNotFound)?;
        execute_pure(
            (linked_application.id.clone(), new_application_id.clone()),
            PrivacyPureRegion::new(|(old_app_id, new_app_id): (i32, i32)| {
                if old_app_id.to_string()[0..3] == new_app_id.to_string()[0..3] {
                    return Err(ServiceError::TooManyFieldsForOnePerson);
                }
                Ok(())
            })
        ).unwrap().transpose()?;

        let mut recipients = Query::get_all_admin_public_keys(db).await?;
        recipients.push(linked_application.public_key.clone());
        recipients.push(pubkey.clone());

        let enc_personal_id_number = EncryptedString::new(
            personal_id_number.clone(),
            &recipients,
        ).await?;

        let candidate = Mutation::update_personal_id(db, candidate.clone(), &enc_personal_id_number.to_owned().into()).await?;
        Ok(
            (candidate, enc_personal_id_number.into())
        )
    }

    pub async fn delete<D: ContextData + Clone>(
        context: Context<D>, 
        db: &DbConn, application: 
        application::Model
    ) -> Result<(), ServiceError> {
        let candidate = ApplicationService::find_related_candidate(db, &application).await?;
        
        let applications = Query::find_applications_by_candidate_id(db, candidate.id.clone()).await?;
        if applications.len() <= 1 &&
            (EncryptedCandidateDetails::from(&candidate).is_filled() ||
            PortfolioService::get_submission_progress(context.clone(), candidate.id.clone())?.index() > 1) {
            return Err(ServiceError::Forbidden);
        }

        Mutation::delete_application(db, application).await?;

        let remaining_applications = Query::find_applications_by_candidate_id(db, candidate.id.clone()).await?;
        if remaining_applications.is_empty() {
            CandidateService::delete_candidate(context, db, candidate).await?;
        }
    
        Ok(())
    }

    fn is_application_id_valid(application_id: i32) -> bool {
        let s = &application_id.to_string();
        if s.len() <= 3 {
            // TODO: does the field of study prefix have to be exactly 6 digits? VYRESIT PODLE PRIHLASEK!!!
            return false;
        }
        let field_of_study_prefix = &s[0..3];
        FIELD_OF_STUDY_PREFIXES.contains(&field_of_study_prefix)
    }

    pub async fn find_related_candidate(
        db: &DbConn,
        application: &application::Model,
    ) -> Result<candidate::Model, ServiceError> {
        let candidate = Query::find_related_candidate(db, application).await?;
        if let Some(candidate) = candidate {
            Ok(candidate)
        } else {
            Err(ServiceError::CandidateNotFound)
        }
    }

    pub async fn add_all_details(
        db: &DbConn,
        application: &application::Model,
        candidate: candidate::Model,
        form: &ApplicationDetails,
    ) -> Result<(candidate::Model, Vec<parent::Model>), ServiceError> {
        let mut recipients = Query::get_all_admin_public_keys_together(db).await?;
        let applications = Query::find_applications_by_candidate_id(db, candidate.id.clone()).await?;
        recipients.append(&mut applications.iter().map(|a| a.public_key.to_owned()).collect());

        let candidate = CandidateService::add_candidate_details(db, candidate, &form.candidate, &recipients, application.id.clone()).await?;
        let parents = ParentService::add_parents_details(db, &candidate, &form.parents, &recipients).await?;
        Ok(
            (
                candidate,
                parents
            )
        )
    }

    pub async fn decrypt_all_details(
        private_key: BBox<String, FakePolicy>,
        db: &DbConn,
        application: &application::Model,
    ) -> Result<ApplicationDetails, ServiceError>  {
        let candidate = ApplicationService::find_related_candidate(db, application).await?;

        let parents = Query::find_candidate_parents(db, &candidate).await?;
        let enc_details = EncryptedApplicationDetails::from((&candidate, &parents));

        if enc_details.is_filled() {
            enc_details.decrypt(private_key).await
        } else {
            Err(ServiceError::Forbidden)
        }
    }

    pub async fn list_applications(
        private_key: &BBox<String, FakePolicy>,
        db: &DbConn,
        field_of_study: Option<String>,
        page: Option<u64>,
        sort: Option<String>,
    ) -> Result<Vec<ApplicationResponse>, ServiceError> {
        let applications = Query::list_applications(db, field_of_study, page, sort).await?;

        futures::future::try_join_all(
            applications
                .iter()
                .map(|c: &crate::database::query::application::ApplicationCandidateJoin| async move {
                    let related_applications = Query::find_applications_by_candidate_id(db, c.candidate_id.clone()).await?.iter()
                        .map(|a| a.id.clone()).collect();
                    ApplicationResponse::from_encrypted(
                        private_key,
                        c.to_owned(),
                        related_applications,
                ).await
                })
        ).await
    }

    async fn decrypt_private_key(
        application: application::Model,
        password: BBox<String, FakePolicy>,
    ) -> Result<BBox<String, FakePolicy>, ServiceError> {
        let private_key_encrypted = application.private_key;

        let private_key = my_decrypt_password(private_key_encrypted, password).await?;

        Ok(private_key.specialize_policy().unwrap())
    }

    pub async fn extend_session_duration_to_14_days(db: &DbConn, session: session::Model) -> Result<session::Model, ServiceError> {
        let now = chrono::Utc::now().naive_utc();
        let result = session.updated_at.clone().into_ppr(PrivacyPureRegion::new(|updated_at: NaiveDateTime| {
            let updated_at = updated_at.checked_add_signed(Duration::days(1)).ok_or(ServiceError::Unauthorized)?;
            if now >= updated_at {
                Result::<_, ServiceError>::Ok(Some(()))
            } else {
                Result::<_, ServiceError>::Ok(None)
            }
        }));

        match result.transpose()?.transpose() {
            Some(_) => {
                let new_expires_at = now.checked_add_signed(Duration::days(14)).ok_or(ServiceError::Unauthorized)?;
                let new_expires_at = BBox::new(new_expires_at, session.updated_at.policy().clone());
                Ok(Mutation::update_session_expiration(db, session, new_expires_at).await?)
            },
            None => Ok(session)
        }
    }

    pub async fn reset_password<D: ContextData + Clone>(
        context: Context<D>,
        admin_private_key: BBox<String, FakePolicy>,
        db: &DbConn,
        id: BBox<i32, FakePolicy>,
    ) -> Result<CreateCandidateResponse, ServiceError> {
        let application = Query::find_application_by_id(db, id.clone()).await?
            .ok_or(ServiceError::CandidateNotFound)?;
        let candidate = ApplicationService::find_related_candidate(db, &application).await?;
       
        let new_password_plain = crypto::random_12_char_string();
        let new_password_hash = crypto::hash_password(new_password_plain.clone()).await?;
        let new_password_hash = BBox::new(new_password_hash, FakePolicy::new());

        let (pubkey, priv_key_plain_text) = crypto::create_identity();
        let encrypted_priv_key = crypto::encrypt_password(
            priv_key_plain_text.clone(),
            new_password_plain.clone()
        ).await?;

        let pubkey = BBox::new(pubkey, FakePolicy::new());
        let encrypted_priv_key = BBox::new(encrypted_priv_key, FakePolicy::new());

        Self::delete_old_sessions(db, &application, 0).await?;
        let application = Mutation::update_application_password_and_keys(db,
             application,
             new_password_hash,
             pubkey,
             encrypted_priv_key,
        ).await?;

        
        // user might no have filled his details yet, but personal id number is filled from beginning
        let personal_id_number = EncryptedString::from(application.personal_id_number.clone())
            .decrypt(&admin_private_key)
            .await?;

        let applications = Query::find_applications_by_candidate_id(db, candidate.id.clone()).await?;
        let mut recipients = vec![]; 
        let mut admin_public_keys = Query::get_all_admin_public_keys_together(db).await?;
        recipients.append(&mut admin_public_keys);
        recipients.append(&mut applications.iter().map(|a| a.public_key.to_owned()).collect());
        
        let candidate = Self::update_all_application_details(db,
             application.id,
             candidate,
             &recipients,
             &admin_private_key
        ).await?;

        if PortfolioService::get_submission_progress(context.clone(), candidate.id.clone())? == SubmissionProgress::Submitted {
            PortfolioService::reencrypt_portfolio(
                context,
                candidate.id,
                admin_private_key,
                &recipients,
            ).await?;
        }

        Ok(
            CreateCandidateResponse {
                application_id: id.into_any_policy(),
                field_of_study: application.field_of_study.into_any_policy(),
                applications: applications.iter()
                    .map(|a| a.id.clone().into_any_policy())
                    .collect(),
                personal_id_number,
                password: BBox::new(new_password_plain, AnyPolicy::new(FakePolicy::new())),
            }
        )
    }

    async fn update_all_application_details(db: &DbConn,
         application_id: BBox<i32, FakePolicy>,
         candidate: candidate::Model,
         recipients: &Vec<BBox<String, FakePolicy>>,
         admin_private_key: &BBox<String, FakePolicy>
    ) -> Result<candidate::Model, ServiceError> {
        let parents = Query::find_candidate_parents(db, &candidate).await?;
        let dec_details = EncryptedApplicationDetails::from((&candidate, &parents))
            .decrypt(admin_private_key.to_owned()).await?;

        let enc_details = EncryptedApplicationDetails::new(&dec_details, recipients).await?;

        let candidate = Mutation::update_personal_id(db,
            candidate,
            &enc_details.candidate.personal_id_number
                .to_owned()
                .ok_or(ServiceError::CandidateDetailsNotSet)?
                .into(),
        ).await?;

        let candidate = Mutation::update_candidate_opt_details(db,
            candidate,
            enc_details.candidate,
            application_id
        ).await?;

        for i in 0..enc_details.parents.len() {
            Mutation::add_parent_details(db, parents[i].clone(), enc_details.parents[i].clone()).await?;
        }
        
        Ok(candidate)
    }
}

#[async_trait]
impl AuthenticableTrait for ApplicationService {
    type User = application::Model;
    type Session = session::Model;

    async fn login(
        db: &DbConn,
        application_id: BBox<i32, FakePolicy>,
        password: BBox<String, FakePolicy>,
        ip_addr: BBox<String, FakePolicy>,
    ) -> Result<(BBox<String, FakePolicy>, BBox<String, FakePolicy>), ServiceError> {
        let application = Query::find_application_by_id(db, application_id)
            .await?
            .ok_or(ServiceError::CandidateNotFound)?;

        let session_id = Self::new_session(db, &application, password.clone(), ip_addr).await?;
        let private_key = Self::decrypt_private_key(application, password).await?;

        Ok((session_id, private_key))
    }

    async fn auth(db: &DbConn, session_uuid: BBox<Uuid, FakePolicy>) -> Result<application::Model, ServiceError> {
        let session = Query::find_session_by_uuid(db, session_uuid)
            .await?
            .ok_or(ServiceError::Unauthorized)?;

        if !SessionService::is_valid(&session).await? {
            Mutation::delete_session(db, session.into_active_model()).await?;
            return Err(ServiceError::ExpiredSession);
        }
        // Candidate authenticated

        Self::extend_session_duration_to_14_days(db, session.clone()).await?;

        let application = Query::find_application_by_id(db, session.candidate_id)
            .await?
            .ok_or(ServiceError::CandidateNotFound)?;

        Ok(application)
    }

    async fn logout(db: &DbConn, session: session::Model) -> Result<(), ServiceError> {
        Mutation::delete_session(db, session.into_active_model()).await?;
        Ok(())
    }

    async fn new_session(
        db: &DbConn,
        application: &application::Model,
        password: BBox<String, FakePolicy>,
        ip_addr: BBox<String, FakePolicy>,
    ) -> Result<BBox<String, FakePolicy>, ServiceError> {
        if !my_verify_password(password.clone(), application.password.clone()).await? {
            return Err(ServiceError::InvalidCredentials);
        }
        // user is authenticated, generate a new session
        let random_uuid: BBox<Uuid, FakePolicy> = BBox::new(Uuid::new_v4(), FakePolicy::new());

        let session = Mutation::insert_candidate_session(db, random_uuid, application.id.clone(), ip_addr).await?;

        Self::delete_old_sessions(db, &application, 3).await?;
        Ok(session.id.into_bbox())
    }
    async fn delete_old_sessions(
        db: &DbConn,
        application: &application::Model,
        keep_n_recent: usize,
    ) -> Result<(), ServiceError> {
        let sessions = Query::find_related_application_sessions(db, &application)
            .await?
            .iter()
            .map(|s| s.to_owned().into_active_model())
            .collect();
        
        SessionService::delete_sessions(db, sessions, keep_n_recent).await?;
        Ok(())
    }
}

#[cfg(test)]
mod application_tests {
    use alohomora::{bbox::BBox, context::Context, pcr::{execute_pcr, PrivacyCriticalRegion}, policy::NoPolicy, pure::{execute_pure, PrivacyPureRegion}, testing::TestContextData};
    use portfolio_policies::{context::ContextDataType, FakePolicy};
    use rocket::figment::util;
    //use sea_orm::sea_query::private;

    use crate::{crypto, models::auth::AuthenticableTrait, services::{application_service::ApplicationService, candidate_service::tests::put_user_data}, utils::{self, db::get_memory_sqlite_connection}};
    use crate::services::admin_service::admin_tests::create_admin;

    #[tokio::test]
    async fn test_application_id_validation() {
        assert!(ApplicationService::is_application_id_valid(101_101));
        assert!(ApplicationService::is_application_id_valid(102_107));
        assert!(ApplicationService::is_application_id_valid(103_109));
        assert!(!ApplicationService::is_application_id_valid(104_109));
        assert!(!ApplicationService::is_application_id_valid(100_109));
        assert!(!ApplicationService::is_application_id_valid(201_109));
        assert!(!ApplicationService::is_application_id_valid(101));
    }

    fn get_test_context() -> Context<TestContextData<ContextDataType>> {
        Context::test(ContextDataType{
            session_id: Some(BBox::new(utils::db::TESTING_ADMIN_COOKIE.to_string(), NoPolicy::new())),
            key: Some(BBox::new(utils::db::TESTING_ADMIN_KEY.to_string(), NoPolicy::new())),
        })
    }

    #[tokio::test]
    async fn test_password_reset() {
        let db = get_memory_sqlite_connection().await;
        let admin = create_admin(&db).await;
        let (application, _, _) = put_user_data(&db).await;

        // The admin's private key.
        let private_key = execute_pcr(admin.private_key, 
            PrivacyCriticalRegion::new(|private_key, _, _|{
                crypto::decrypt_password(private_key, "admin".to_string())
            }),
        ()).unwrap().await.unwrap();

        // The ip and password for the login happens with.
        let ip = BBox::new("127.0.0.1".to_string(), FakePolicy::new());
        let password = BBox::new("test".to_string(), FakePolicy::new());
        assert!(
            ApplicationService::login(&db, application.id.clone(), password.clone(), ip.clone()).await.is_ok()
        );

        let new_password = ApplicationService::reset_password(
            get_test_context(),
            BBox::new(private_key, FakePolicy::new()),
            &db,
            application.id.clone()
        ).await
            .unwrap()
            .password;
        assert!(
            ApplicationService::login(&db, application.id.clone(), password, ip.clone()).await.is_err()
        );
        assert!(
            ApplicationService::login(&db, application.id, new_password.specialize_policy().unwrap(), ip.clone()).await.is_ok()
        );
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_private_key_with_passphrase() {
        let db = get_memory_sqlite_connection().await;

        let plain_text_password = "test".to_string();

        let secret_message = "trnka".to_string();

        let application = ApplicationService::create(
            get_test_context(),
            &BBox::new("".to_string(), FakePolicy::new()),
            &db,
            BBox::new(103100, FakePolicy::new()),
            &BBox::new(plain_text_password.clone(), FakePolicy::new()),
            BBox::new("".to_string(), FakePolicy::new())
        ).await.unwrap().0;

        let public_key = execute_pcr(application.public_key, 
            PrivacyCriticalRegion::new(|public_key: String, _, _| {
                public_key
            }), ()).unwrap();

        // ideally we'd do things this way but we cant await outside pcr bc public_key doesn't live long enough
        // and we need it to stick around
        // let encrypted_message = execute_pcr(application.public_key, 
        //     PrivacyCriticalRegion::new(|public_key: String, _, _| {
        //         crypto::encrypt_password_with_recipients(&secret_message, &vec![&public_key])
        //     }), ()).unwrap().await.unwrap();

        let encrypted_message = crypto::encrypt_password_with_recipients(&secret_message, &vec![&public_key]).await.unwrap();

        let private_key_plain_text = execute_pcr(application.private_key.clone(), 
            PrivacyCriticalRegion::new(|private_key: String, _, _| {
                crypto::decrypt_password(private_key, plain_text_password.clone())
            }), ()).unwrap().await.unwrap();

        let decrypted_message = execute_pcr(application.private_key, 
            PrivacyCriticalRegion::new(|private_key: String, _, _| {
                crypto::decrypt_password_with_private_key(&encrypted_message, &private_key_plain_text)
            }), ()).unwrap().await.unwrap();

        assert_eq!(secret_message, decrypted_message);
    }
}