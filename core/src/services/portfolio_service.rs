use std::{path::{Path, PathBuf}};
use age::x25519::Recipient;

use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::pcr::PrivacyCriticalRegion;
use entity::candidate;
use log::{info, warn};
use alohomora::policy::Policy;
use alohomora::unbox::unbox;
use sea_orm::{DbConn};
use serde::{Serialize, ser::{SerializeStruct}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use portfolio_policies::FakePolicy;

use crate::{error::ServiceError, Query, crypto};
use crate::crypto_helpers::{get_context};
use portfolio_policies::context::ContextDataType;

#[derive(Debug, PartialEq)]
pub enum SubmissionProgress {
    NoneInCache,
    SomeInCache(Vec<FileType>),
    AllInCache,
    Submitted,
}

impl SubmissionProgress {
    pub fn index(&self) -> usize {
        match self {
            SubmissionProgress::NoneInCache => 1,
            SubmissionProgress::SomeInCache(_) => 2,
            SubmissionProgress::AllInCache => 3,
            SubmissionProgress::Submitted => 4,
        }
    }
}

// Serialize the enum so that the JSON contains status field and a list of files present in cache
impl Serialize for SubmissionProgress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut progress = serializer.serialize_struct("SubmissionProgress", 2)?;
        progress.serialize_field("status", &self.index())?;

        match self {
            SubmissionProgress::SomeInCache(files) => {
                progress.serialize_field("files", files)?;
            }
            _ => {
                progress.serialize_field("files", &Vec::<FileType>::new())?;
            }
        };

        progress.end()
    }
}


#[derive(Debug, Copy, PartialEq, Clone)]
pub enum FileType {
    CoverLetterPdf = 1,
    PortfolioLetterPdf = 2,
    PortfolioZip = 3,
    Age = 4,
}

impl FileType {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::CoverLetterPdf => "MOTIVACNI_DOPIS.pdf",
            FileType::PortfolioLetterPdf => "PORTFOLIO.pdf",
            FileType::PortfolioZip => "PORTFOLIO.zip",
            FileType::Age => "PORTFOLIO.age",
        }
    }

    pub fn iter_cache() -> impl Iterator<Item = Self> {
        [
            FileType::CoverLetterPdf,
            FileType::PortfolioLetterPdf,
            FileType::PortfolioZip,
        ]
        .iter()
        .copied()
    }
}

impl ToString for FileType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.index() as u32)
    }
}


pub struct PortfolioService;
impl PortfolioService {
    pub fn get_submission_progress<P: Policy>(
        candidate_id: BBox<i32, P>
    ) -> Result<SubmissionProgress, ServiceError> {
        candidate_id.into_unbox(get_context(), PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
            Self::get_submission_progress_raw(candidate_id)
        }), ()).unwrap_or(Err(ServiceError::PolicyCheckFailed))
    }

    fn get_submission_progress_raw(candidate_id: i32) -> Result<SubmissionProgress, ServiceError> {
        let path = Self::get_file_store_path().join(&candidate_id.to_string());
        if !path.exists() {
            return Err(ServiceError::CandidateNotFound);
        }
        let cache_path = path.join("cache");

        if path.join(FileType::Age.as_str()).exists() {
            return Ok(SubmissionProgress::Submitted);
        }

        let mut files = Vec::new();
        for file in FileType::iter_cache() {
            if cache_path.join(file.as_str()).exists() {
                files.push(file);
            }
        }
        match files.len() {
            0 => Ok(SubmissionProgress::NoneInCache),
            3 => Ok(SubmissionProgress::AllInCache),
            _ => Ok(SubmissionProgress::SomeInCache(files)),
        }
    }


    // Get root path or local directory
    fn get_file_store_path() -> PathBuf {
        dotenv::dotenv().ok();
        Path::new(&std::env::var("PORTFOLIO_STORE_PATH").unwrap_or_else(|_| "".to_string())).to_path_buf()
    }

    /// Writes file to desired location
    async fn write_portfolio_file(
        candidate_id: i32,
        data: Vec<u8>,
        filename: FileType,
    ) -> Result<(), ServiceError> {
        info!("PORTFOLIO {} CACHE {} WRITE STARTED", candidate_id, filename.as_str());

        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");

        let mut file = tokio::fs::File::create(cache_path.join(filename.as_str())).await?;

        file.write_all(&data).await?;

        info!("PORTFOLIO {} CACHE {} WRITE FINISHED", candidate_id, filename.as_str());
        Ok(())
    }

    pub async fn create_user_dir(application_id: BBox<i32, FakePolicy>) -> tokio::io::Result<()> {
        application_id.into_unbox(get_context(), PrivacyCriticalRegion::new(|application_id: i32, ()| {
            tokio::fs::create_dir_all(
                Self::get_file_store_path()
                    .join(&application_id.to_string())
                    .join("cache")
            )
        }), ()).unwrap().await
    }

    
    pub async fn add_cover_letter_to_cache<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P1>,
        letter: BBox<Vec<u8>, P2>,
    ) -> Result<(), ServiceError> {
        match unbox(
            (candidate_id, letter),
            context,
            PrivacyCriticalRegion::new(|(candidate_id, letter): (i32, Vec<u8>), ()| {
                Self::write_portfolio_file(candidate_id, letter, FileType::CoverLetterPdf)
            }),
            ()
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }

    pub async fn add_portfolio_letter_to_cache<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P1>,
        letter: BBox<Vec<u8>, P2>,
    ) -> Result<(), ServiceError> {
        match unbox(
            (candidate_id, letter),
            context,
            PrivacyCriticalRegion::new(|(candidate_id, letter): (i32, Vec<u8>), ()| {
                Self::write_portfolio_file(candidate_id, letter, FileType::PortfolioLetterPdf)
            }),
            ()
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }

    pub async fn add_portfolio_zip_to_cache<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P1>,
        letter: BBox<Vec<u8>, P2>,
    ) -> Result<(), ServiceError> {
        match unbox(
            (candidate_id, letter),
            context,
            PrivacyCriticalRegion::new(|(candidate_id, letter): (i32, Vec<u8>), ()| {
                Self::write_portfolio_file(candidate_id, letter, FileType::PortfolioZip)
            }),
            ()
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }
    
    
    pub async fn is_cover_letter(candidate_id: i32) -> bool {
        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");
        
        tokio::fs::metadata(cache_path.join(cache_path.join(FileType::CoverLetterPdf.as_str())))
        .await
        .is_ok()
    }

    pub async fn is_portfolio_letter(candidate_id: i32) -> bool {
        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");

        tokio::fs::metadata(
            cache_path.join(
                cache_path.join(FileType::PortfolioLetterPdf.as_str())
            )
        )
            .await
            .is_ok()
    }

    pub async fn is_portfolio_zip(candidate_id: i32) -> bool {
        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");

        tokio::fs::metadata(
            cache_path.join(
                cache_path.join(FileType::PortfolioZip.as_str())
            )
        )
            .await
            .is_ok()
    }


    /// Returns true if portfolio is ready to be moved to the final directory
    async fn is_portfolio_prepared(candidate_id: i32) -> bool {
        Self::get_submission_progress_raw(candidate_id).ok() == Some(SubmissionProgress::AllInCache)
    }

    // Delete single item from cache
    pub async fn delete_cache_item(candidate_id: i32, file_type: FileType) -> Result<(), ServiceError> {
        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");

        tokio::fs::remove_file(cache_path.join(file_type.as_str())).await?;

        Ok(())
    }

    pub async fn delete_cover_letter_from_cache<P: Policy>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P>,
    ) -> Result<(), ServiceError> {
        match candidate_id.into_unbox(
            context,
            PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
              Self::delete_cache_item(candidate_id,  FileType::CoverLetterPdf)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }

    pub async fn delete_portfolio_letter_from_cache<P: Policy>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P>,
    ) -> Result<(), ServiceError> {
        match candidate_id.into_unbox(
            context,
            PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
              Self::delete_cache_item(candidate_id,  FileType::PortfolioLetterPdf)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }

    pub async fn delete_portfolio_zip_from_cache<P: Policy>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P>,
    ) -> Result<(), ServiceError> {
        match candidate_id.into_unbox(
            context,
            PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
              Self::delete_cache_item(candidate_id,  FileType::PortfolioZip)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await,
        }
    }

    /// Removes all files from cache
    pub async fn delete_cache(candidate_id: i32) -> Result<(), ServiceError> {
        let cache_path = Self::get_file_store_path().join(&candidate_id.to_string()).join("cache");
        tokio::fs::remove_dir_all(&cache_path).await?;
        // Recreate blank cache directory
        tokio::fs::create_dir_all(&cache_path).await?;

        Ok(())
    }

    // First PCR for submit.
    async fn submit_pcr_1(candidate_id: i32) -> Result<(), ServiceError> {
        let path = Self::get_file_store_path().join(&candidate_id.to_string()).to_path_buf();
        let cache_path = path.join("cache");

        if Self::is_portfolio_prepared(candidate_id).await == false {
            return Err(ServiceError::IncompletePortfolio);
        }

        info!("PORTFOLIO {} SUBMIT STARTED", candidate_id);

        let mut archive = tokio::fs::File::create(path.join(FileType::PortfolioZip.as_str())).await?;
        let mut writer = async_zip::tokio::write::ZipFileWriter::with_tokio(&mut archive);
        let mut buffer = vec![vec![], vec![], vec![]];

        let filenames = vec![FileType::CoverLetterPdf, FileType::PortfolioLetterPdf, FileType::PortfolioZip];
        for (index, entry) in buffer.iter_mut().enumerate() {
            let filename = filenames[index];
            let mut entry_file = tokio::fs::File::open(cache_path.join(filename.as_str())).await?;

            entry_file.read_to_end(entry).await?;
        }

        Self::delete_cache(candidate_id).await?;

        for (index, entry) in buffer.iter_mut().enumerate() {
            let filename = filenames[index];
            let builder = async_zip::ZipEntryBuilder::new(
                filename.to_string().into(),
                async_zip::Compression::Deflate,
            );

            writer.write_entry_whole(builder, &entry).await?;
        }

        writer.close().await?;
        archive.shutdown().await?;
        Ok(())
    }

    // Second submit PCR.
    async fn submit_pcr_2(candidate_id: i32, recipients: Vec<String>) -> Result<(), ServiceError> {
        let path = Self::get_file_store_path().join(&candidate_id.to_string()).to_path_buf();
        let final_path = path.join(FileType::PortfolioZip.as_str());
        crypto::encrypt_file_with_recipients(
            &final_path,
            &final_path.with_extension("age"),
            recipients,
        ).await?;
        tokio::fs::remove_file(final_path).await?;

        if !Self::is_portfolio_submitted(candidate_id).await {
            return Err(ServiceError::PortfolioWriteError)
        }

        Ok(())
    }

    /// Move files from cache to final directory and delete cache afterwards
    pub async fn submit(candidate: &candidate::Model, db: &DbConn) -> Result<(), ServiceError> {
        match candidate.id.clone().into_unbox(
            get_context(),
            PrivacyCriticalRegion::new(move |candidate_id: i32, ()| {
                Self::submit_pcr_1(candidate_id)
            }),
            ()
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed)?,
            Ok(result) => result.await?,
        }

        let mut applications_pubkeys = Query::find_applications_by_candidate_id(db, candidate.id.clone())
            .await?
            .iter()
            .map(|a| a.public_key.clone()).collect();

        let mut admin_public_keys = Query::get_all_admin_public_keys_together(db).await?;
        let mut recipients = vec![];
        recipients.append(&mut admin_public_keys);
        recipients.append(&mut applications_pubkeys);

        // Privacy Critical region.
        match unbox(
            (candidate.id.clone(), recipients),
            get_context(),
            PrivacyCriticalRegion::new(|(candidate_id, recipients): (i32, Vec<String>), ()| {
                Self::submit_pcr_2(candidate_id, recipients)
            }),
            ()
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed)?,
            Ok(result) => result.await,
        }
    }

    /// Delete PORTFOLIO.age file
    async fn delete_portfolio_pcr(candidate_id: i32) -> Result<(), ServiceError> {
        let path = Self::get_file_store_path().join(&candidate_id.to_string()).to_path_buf();

        let portfolio_path = path.join(FileType::PortfolioZip.as_str());
        let portfolio_age_path = portfolio_path.with_extension("age");

        if tokio::fs::metadata(&portfolio_path).await.is_ok() {
            tokio::fs::remove_file(&portfolio_path).await?;
        }

        if tokio::fs::metadata(&portfolio_age_path).await.is_ok() {
            tokio::fs::remove_file(&portfolio_age_path).await?;
        }
        Ok(())
    }
    pub async fn delete_portfolio<P: Policy>(candidate_id: BBox<i32, P>) -> Result<(), ServiceError> {
        match candidate_id.into_unbox(
            get_context(),
            PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
                Self::delete_portfolio_pcr(candidate_id)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => {
                result.await?;
                Ok(())
            },
        }
    }

    /// Deletes all candidate folder. Used ONLY when candidate is deleted!
    pub async fn delete_candidate_root<P: Policy>(candidate_id: BBox<i32, P>) -> Result<(), ServiceError> {
        match candidate_id.into_unbox(
            get_context(),
            PrivacyCriticalRegion::new(|candidate_id: i32, ()| {
                let path = Self::get_file_store_path().join(&candidate_id.to_string()).to_path_buf();
                tokio::fs::remove_dir_all(path)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => {
                result.await?;
                Ok(())
            },
        }
    }

    /// Returns true if portfolio is submitted
    pub async fn is_portfolio_submitted(candidate_id: i32) -> bool {
        let path = Self::get_file_store_path().join(&candidate_id.to_string()).to_path_buf();

        tokio::fs::metadata(path.join(FileType::Age.as_str())).await.is_ok()
    }

    /// Returns decrypted portfolio zip as Vec of bytes
    pub async fn get_portfolio<P1: Policy + Clone+ 'static, P2: Policy + Clone + 'static>(
        context: Context<ContextDataType>,
        candidate_id: BBox<i32, P1>,
        private_key: BBox<String, P2>,
    ) -> Result<Vec<u8>, ServiceError> {
        match unbox(
            (candidate_id, private_key),
            context,
            PrivacyCriticalRegion::new(|(candidate_id, private_key): (i32, String), ()| {
                let path = Self::get_file_store_path()
                    .join(&candidate_id.to_string())
                    .join(FileType::Age.as_str())
                    .to_path_buf();
                crypto::decrypt_file_with_private_key_as_buffer(path, private_key)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed),
            Ok(result) => result.await
        }
    }

    // PCR for reencrypt_portfolio.
    async fn reencrypt_portfolio_pcr(candidate_id: i32, private_key: String, recipients: Vec<String>) -> Result<(), ServiceError> {
        let path = Self::get_file_store_path()
            .join(&candidate_id.to_string())
            .join(FileType::Age.as_str())
            .to_path_buf();

        let plain_portfolio = crypto::decrypt_file_with_private_key_as_buffer(
            path.to_owned(),
            private_key
        ).await?;

        let enc_portfolio= crypto::encrypt_buffer_with_recipients(
            &plain_portfolio,
            &recipients
        ).await?;

        tokio::fs::remove_file(path.to_owned()).await?;

        tokio::fs::write(path, enc_portfolio).await?;

        Ok(())
    }

    pub async fn reencrypt_portfolio(candidate_id: BBox<i32, FakePolicy>,
        private_key: BBox<String, FakePolicy>,
        recipients: &Vec<BBox<String, FakePolicy>>,
    ) -> Result<(), ServiceError> {
        match unbox(
            (candidate_id.clone(), private_key, recipients.clone()),
            get_context(),
            PrivacyCriticalRegion::new(|(candidate_id, private_key, recipients): (i32, String, Vec<String>), ()| {
                Self::reencrypt_portfolio_pcr(candidate_id, private_key, recipients)
            }),
            (),
        ) {
            Err(_) => Err(ServiceError::PolicyCheckFailed)?,
            Ok(result) => result.await,
        }
    }
}

#[cfg(test)]
mod tests {
    use alohomora::{bbox::BBox, pcr::execute_pcr};
    use serial_test::serial;

    use crate::{services::{portfolio_service::{PortfolioService, FileType}, candidate_service::{CandidateService, tests::put_user_data}}, utils::db::get_memory_sqlite_connection, crypto};
    use std::path::PathBuf;
    use alohomora::pcr::PrivacyCriticalRegion;
    use alohomora::policy::Policy;
    use portfolio_policies::FakePolicy;
    use crate::crypto_helpers::get_context;

    const APPLICATION_ID: i32 = 103151;

    #[cfg(test)]
    fn open<T, P: Policy>(bbox: BBox<T, P>) -> T {
        bbox.into_unbox(get_context(), PrivacyCriticalRegion::new(|t: T, ()| t), ()).unwrap()
    }

    #[cfg(test)]
    async fn create_data_store_temp_dir(application_id: i32) -> (PathBuf, PathBuf, PathBuf) {
        let random_number: u32 = rand::Rng::gen(&mut rand::thread_rng());
        
        let temp_dir = std::env::temp_dir().join("portfolio_test_tempdir").join(random_number.to_string());
        let application_dir = temp_dir.join(application_id.to_string());
        let application_cache_dir = application_dir.join("cache");

        tokio::fs::create_dir_all(application_cache_dir.clone()).await.unwrap();

        std::env::set_var("PORTFOLIO_STORE_PATH", temp_dir.to_str().unwrap());

        (temp_dir, application_dir, application_cache_dir)
    }

    #[cfg(test)]
    async fn clear_data_store_temp_dir(temp_dir: PathBuf) {
        tokio::fs::remove_dir_all(temp_dir).await.unwrap();

        std::env::remove_var("PORTFOLIO_STORE_PATH");
    }

    #[tokio::test]
    #[serial]
    async fn test_folder_creation() {
        let db = get_memory_sqlite_connection().await;

        let temp_dir = std::env::temp_dir().join("portfolio_test_tempdir").join("create_folder");
        std::env::set_var("PORTFOLIO_STORE_PATH", temp_dir.to_str().unwrap());

        let candidate = CandidateService::create(&db, BBox::new("".to_string(), FakePolicy::new()))
            .await
            .ok()
            .unwrap();

        let candidate_id = open(candidate.id.clone());
        assert!(tokio::fs::metadata(temp_dir.join(candidate_id.to_string())).await.is_ok());
        assert!(tokio::fs::metadata(temp_dir.join(candidate_id.to_string()).join("cache")).await.is_ok());

        tokio::fs::remove_dir_all(temp_dir).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_write_portfolio_file() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::write_portfolio_file(APPLICATION_ID, vec![0], crate::services::portfolio_service::FileType::PortfolioLetterPdf).await.unwrap();
        
        assert!(tokio::fs::metadata(application_cache_dir.join(FileType::PortfolioLetterPdf.as_str())).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_add_cover_letter_to_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_cover_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(tokio::fs::metadata(application_cache_dir.join("MOTIVACNI_DOPIS.pdf")).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_cover_letter_from_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        let context = todo!();

        PortfolioService::add_cover_letter_to_cache(context, BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        PortfolioService::delete_cover_letter_from_cache(context, BBox::new(APPLICATION_ID, FakePolicy::new())).await.unwrap();

        assert!(tokio::fs::metadata(application_cache_dir.join("MOTIVACNI_DOPIS.pdf")).await.is_err());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_is_cover_letter() {
        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_cover_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(PortfolioService::is_cover_letter(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_cache_item() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_cover_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        PortfolioService::delete_cache_item(APPLICATION_ID, FileType::CoverLetterPdf).await.unwrap();

        assert!(tokio::fs::metadata(application_cache_dir.join("MOTIVACNI_DOPIS.pdf")).await.is_err());
        
        clear_data_store_temp_dir(temp_dir).await;
    }
    

    #[tokio::test]
    #[serial]
    async fn test_add_portfolio_letter_to_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;
        
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(tokio::fs::metadata(application_cache_dir.join("PORTFOLIO.pdf")).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_portfolio_letter_from_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        PortfolioService::delete_portfolio_letter_from_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new())).await.unwrap();

        assert!(tokio::fs::metadata(application_cache_dir.join("PORTFOLIO.pdf")).await.is_err());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_is_portfolio_letter() {
        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(PortfolioService::is_portfolio_letter(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_add_portfolio_zip_to_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(tokio::fs::metadata(application_cache_dir.join("PORTFOLIO.zip")).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_portfolio_zip_from_cache() {
        let (temp_dir, _, application_cache_dir) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        PortfolioService::delete_portfolio_zip_from_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new())).await.unwrap();

        assert!(tokio::fs::metadata(application_cache_dir.join("PORTFOLIO.zip")).await.is_err());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_is_portfolio_zip() {
        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(PortfolioService::is_portfolio_zip(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_is_portfolio_prepared() {
        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_cover_letter_to_cache(todo!(),  BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        assert!(PortfolioService::is_portfolio_prepared(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;

        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_cover_letter_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        //BBox::new(APPLICATION_ID, FakePolicy::new())
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        assert!(!PortfolioService::is_portfolio_prepared(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_cache() {
        let (temp_dir, _, _) = create_data_store_temp_dir(APPLICATION_ID).await;

        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(APPLICATION_ID, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        
        assert!(PortfolioService::is_portfolio_zip(APPLICATION_ID).await);

        PortfolioService::delete_cache(APPLICATION_ID).await.unwrap();

        assert!(!PortfolioService::is_portfolio_zip(APPLICATION_ID).await);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_add_portfolio() {
        let db = get_memory_sqlite_connection().await;
        let (_, candidate, _) = put_user_data(&db).await;
        let candidate_id = open(candidate.id.clone());

        let (temp_dir, application_dir, _) = create_data_store_temp_dir(candidate_id).await;

        PortfolioService::add_cover_letter_to_cache(todo!(),  BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        PortfolioService::submit(&candidate.clone(), &db).await.unwrap();
        
        assert!(tokio::fs::metadata(application_dir.join("PORTFOLIO.age")).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_portfolio() {
        let db = get_memory_sqlite_connection().await;
        let (_, candidate, _) = put_user_data(&db).await;
        let candidate_id = open(candidate.id.clone());

        let (temp_dir, application_dir, _) = create_data_store_temp_dir(candidate_id).await;

        PortfolioService::add_cover_letter_to_cache(todo!(),  BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        PortfolioService::submit(&candidate, &db).await.unwrap();
        
        assert!(tokio::fs::metadata(application_dir.join("PORTFOLIO.age")).await.is_ok());

        PortfolioService::delete_portfolio(candidate.id).await.unwrap();

        assert!(!tokio::fs::metadata(application_dir.join("PORTFOLIO.age")).await.is_ok());

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_is_portfolio_submitted() {
        let db = get_memory_sqlite_connection().await;

        let (_, candidate, _) = put_user_data(&db).await;
        let candidate_id = open(candidate.id.clone());

        let (temp_dir, _, _) = create_data_store_temp_dir(candidate_id).await;

        PortfolioService::add_cover_letter_to_cache(todo!(),  BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        PortfolioService::submit(&candidate, &db).await.unwrap();
        
        assert!(PortfolioService::is_portfolio_submitted(candidate_id).await);

        clear_data_store_temp_dir(temp_dir).await;

        let (temp_dir, application_dir, _) = create_data_store_temp_dir(candidate_id).await;

        PortfolioService::add_cover_letter_to_cache(todo!(),  BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new())).await.unwrap();

        PortfolioService::submit(&candidate, &db).await.unwrap();

        tokio::fs::remove_file(application_dir.join("PORTFOLIO.age")).await.unwrap();
        
        let is_submitted = execute_pcr(candidate.id, 
            PrivacyCriticalRegion::new(|id, _, _|{
                PortfolioService::is_portfolio_submitted(id)
            }), ()).unwrap().await;
        assert!(!is_submitted);

        clear_data_store_temp_dir(temp_dir).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_portfolio() {
        let db = get_memory_sqlite_connection().await;
        let (application, candidate, _parent) = put_user_data(&db).await;
        let candidate_id = open(candidate.id.clone());

        let (temp_dir, _, _) = create_data_store_temp_dir(candidate_id).await;

        let private_key = execute_pcr(application.private_key, 
            PrivacyCriticalRegion::new(|pk, _, _|{pk}), ()).unwrap();

        let private_key = crypto::decrypt_password(private_key, "test".to_string())
            .await
            .unwrap();

        PortfolioService::add_cover_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new()))
            .await
            .unwrap();
        PortfolioService::add_portfolio_letter_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new()))
            .await
            .unwrap();
        PortfolioService::add_portfolio_zip_to_cache(todo!(), BBox::new(candidate_id, FakePolicy::new()), BBox::new(vec![0], FakePolicy::new()))
            .await
            .unwrap();

        PortfolioService::submit(&candidate, &db)
            .await
            .unwrap();

        PortfolioService::get_portfolio(todo!(), candidate.id, BBox::new(private_key, FakePolicy::new()))
            .await
            .unwrap();

        clear_data_store_temp_dir(temp_dir).await;
    }
}
