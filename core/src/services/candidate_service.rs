use alohomora::{bbox::BBox, context::Context};
use entity::candidate;
use sea_orm::DbConn;
use portfolio_policies::FakePolicy;

use crate::{
    models::{candidate_details::EncryptedCandidateDetails, candidate::CandidateDetails},
    error::ServiceError,
    Mutation,
};

use super::{portfolio_service::PortfolioService};

pub struct CandidateService;

impl CandidateService {
    /// Creates a new candidate with:
    /// Encrypted personal identification number
    /// Hashed password
    /// Encrypted private key
    /// Public key
    pub(in crate::services) async fn create<D: alohomora::context::ContextData + Clone>(
        context: Context<D>,
        db: &DbConn,
        enc_personal_id_number: BBox<String, FakePolicy>,
    ) -> Result<candidate::Model, ServiceError> {
        let candidate = Mutation::create_candidate(
            db,
            enc_personal_id_number,
        )
            .await?;

        PortfolioService::create_user_dir(context, candidate.id.clone()).await?;


        Ok(candidate)
    }

    pub async fn delete_candidate<D: alohomora::context::ContextData + Clone>(
        context: Context<D>, 
        db: &DbConn, candidate: 
        candidate::Model
    ) -> Result<(), ServiceError> {
        PortfolioService::delete_candidate_root(context, candidate.id.clone()).await?;

        Mutation::delete_candidate(db, candidate).await?;
        Ok(())
    }

    pub(in crate::services) async fn add_candidate_details(
        db: &DbConn,
        candidate: candidate::Model,
        details: &CandidateDetails,
        recipients: &Vec<BBox<String, FakePolicy>>,
        encrypted_by: BBox<i32, FakePolicy>,
    ) -> Result<entity::candidate::Model, ServiceError> {
        let enc_details = EncryptedCandidateDetails::new(&details, recipients).await?;
        let model = Mutation::update_candidate_opt_details(
            db,
            candidate,
            enc_details,
            encrypted_by
        ).await?;
        Ok(model)
    }
}

#[cfg(test)]
pub mod tests {
    use alohomora::bbox::BBox;
    use alohomora::context::Context;
    use alohomora::pcr::{execute_pcr, PrivacyCriticalRegion};
    use portfolio_policies::context::ContextDataType;
    use sea_orm::DbConn;

    use crate::models::candidate_details::tests::assert_all_application_details;
    use crate::services::admin_service::admin_tests::create_admin;
    use crate::utils::db::get_memory_sqlite_connection;
    use crate::{crypto};

    use crate::models::candidate_details::EncryptedApplicationDetails;
    use entity::{application, candidate, parent};
    use portfolio_policies::FakePolicy;

    use crate::services::application_service::ApplicationService;

    const APPLICATION_ID: i32 = 103151;

    fn get_test_context() -> Context<ContextDataType> {
        Context::empty()
    }

    #[tokio::test]
    async fn test_list_applications() {
        let db = get_memory_sqlite_connection().await;
        let admin = create_admin(&db).await;
        let private_key = execute_pcr(admin.private_key, 
            PrivacyCriticalRegion::new(|private_key: String, _, _| {
                crypto::decrypt_password(private_key, "admin".to_string())
            }), ()).unwrap().await.unwrap();
        //let private_key = crypto::decrypt_password(admin.private_key.discard_box(), "admin".to_string()).await.unwrap();
        let private_key = BBox::new(private_key, FakePolicy::new());
        let candidates = ApplicationService::list_applications(&private_key, &db, None, None, None).await.unwrap();
        assert_eq!(candidates.len(), 0);

        put_user_data(&db).await;

        let candidates = ApplicationService::list_applications(&private_key, &db, None, None, None).await.unwrap();
        assert_eq!(candidates.len(), 1);
    }

    #[cfg(test)]
    pub async fn put_user_data(db: &DbConn) -> (application::Model, candidate::Model, Vec<parent::Model>) {
        use crate::{models::candidate_details::tests::APPLICATION_DETAILS, services::parent_service::ParentService};

        let plain_text_password = "test".to_string();
        let application = ApplicationService::create(
            get_test_context(),
            &BBox::new("".to_string(), FakePolicy::new()),
            db,
            BBox::new(APPLICATION_ID, FakePolicy::new()),
            &BBox::new(plain_text_password, FakePolicy::new()),
            BBox::new("0000001111".to_string(), FakePolicy::new())
        ).await.unwrap().0;

        let candidate= ApplicationService::find_related_candidate(db, &application).await.unwrap();
        ParentService::create(db, candidate.id.clone()).await.unwrap();

        let form = APPLICATION_DETAILS.lock().unwrap().clone();

        let (candidate, parents) = ApplicationService::add_all_details(&db,  &application, candidate, &form)
            .await
            .unwrap();

        (
            application,
            candidate,
            parents,
        )
    }

    #[tokio::test]
    async fn test_put_user_data() {
        let db = get_memory_sqlite_connection().await;
        let (_, candidate, parents) = put_user_data(&db).await;
        assert!(candidate.name.is_some());
        assert!(parents[0].name.is_some());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_user_data() {
        let password = "test".to_string();
        let db = get_memory_sqlite_connection().await;
        let (application, enc_candidate, enc_parent) = put_user_data(&db).await;

        let dec_priv_key = execute_pcr(application.private_key, 
            PrivacyCriticalRegion::new(|private_key: String, _, _| {
                crypto::decrypt_password(private_key, password)
            }), ()).unwrap().await.unwrap();
        // let dec_priv_key = crypto::decrypt_password(application.private_key.clone().discard_box(), password)
        //     .await
        //     .unwrap();
        let dec_priv_key = BBox::new(dec_priv_key, FakePolicy::new());
        let enc_details = EncryptedApplicationDetails::try_from((&enc_candidate, &enc_parent))
            .ok()
            .unwrap();
        let dec_details = enc_details.decrypt(dec_priv_key).await.ok().unwrap();

        assert_all_application_details(&dec_details);
    }
}
