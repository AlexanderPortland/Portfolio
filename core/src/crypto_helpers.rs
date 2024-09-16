use std::path::Path;
use age::decryptor::RecipientsDecryptor;
use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::pcr::{execute_pcr, PrivacyCriticalRegion, Signature};
use alohomora::policy::{AnyPolicy, NoPolicy, Policy, PolicyAnd};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora::testing::TestContextData;
use alohomora::unbox;
use argon2::Argon2;
use futures::channel::mpsc::Receiver;
use portfolio_policies::key::KeyPolicy;
use crate::error::ServiceError;

pub async fn my_hash_password<P: Policy + Clone + 'static>(password_plain_text: BBox<String, P>) -> Result<BBox<String, P>, ServiceError> {
    let hash_res = execute_pcr(password_plain_text.clone(), 
        PrivacyCriticalRegion::new(|plain_text, _, _|{
            crate::crypto::hash_password(plain_text)
        }, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await;

    match hash_res {
        Ok(hash) => Ok(BBox::new(hash, password_plain_text.policy().clone())),
        Err(e) => Err(e),
    }
}

pub async fn my_encrypt_password<P: Policy + Clone + 'static>(
    password_plain_text: BBox<String, KeyPolicy>,
    key: BBox<String, P>
) -> Result<BBox<String, KeyPolicy>, ServiceError> {
    let enc_res = execute_pcr((key.clone(), password_plain_text.clone()), 
        PrivacyCriticalRegion::new(|(key, password), _, _|{
            crate::crypto::encrypt_password(password, key)
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await;

    match enc_res {
        Ok(enc) => Ok(BBox::new(enc, password_plain_text.policy().to_owned())),
        Err(e) => Err(e),
    }
}

// TODO: (aportlan) the fricking args are reversed (what the heck)
pub async fn my_decrypt_password<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    ciphertext: BBox<String, P1>, key: BBox<String, P2>
) -> Result<BBox<String, P1>, ServiceError> {
    let dec_res = execute_pcr((ciphertext.clone(), key.clone()), 
        PrivacyCriticalRegion::new(|(ciphertext, key), _, _|{
            crate::crypto::decrypt_password(ciphertext, key)
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await?;

    Ok(BBox::new(dec_res, ciphertext.policy().to_owned()))
}

async fn dumb_helper2<P: Policy>(password_plain_text: String, recipients: Vec<String>, combined_policy: P) -> Result<BBox<String, P>, ServiceError> {
        let recipients_ref = recipients.iter().map(|s|s.as_str()).collect::<Vec<&str>>();

        let dec_res = crate::crypto::encrypt_password_with_recipients(&password_plain_text, &recipients_ref).await?;
        Ok(BBox::new(dec_res, combined_policy))
}

pub async fn my_encrypt_password_with_recipients<P: Policy + Clone + 'static>(
    password_plain_text: BBox<String, P>,
    recipients: &Vec<BBox<String, NoPolicy>>,
) -> Result<BBox<String, P>, ServiceError> {
    // TODO: (aportlan) fix this shit
    let r = execute_pcr((password_plain_text.clone(), recipients.clone()), 
    PrivacyCriticalRegion::new(|(plaintext, recipients): (String, Vec<String>), _, _|{
        dumb_helper2(plaintext, recipients, password_plain_text.policy().clone())
    },
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await;

    r
}

async fn dumb_helper<P: Policy>(password_encrypted: String, unboxed_key: String, combined_policy: P) -> Result<BBox<String, P>, ServiceError> {
    let dec = crate::crypto::decrypt_password_with_private_key(&password_encrypted, &unboxed_key).await?;
    Ok(BBox::new(dec, combined_policy))
}

pub async fn my_decrypt_password_with_private_key<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    password_encrypted: BBox<String, P1>,
    key: BBox<String, P2>,
) -> Result<BBox<String, P1>, ServiceError> {
    let policy = password_encrypted.policy().clone();
    execute_pcr(
        (password_encrypted, key), 
        PrivacyCriticalRegion::new( 
            |(unboxed_password_encrypted, unboxed_key): (String, String), combined_policy, _| {
                    dumb_helper(unboxed_password_encrypted, unboxed_key, policy)
                },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await
    
}

pub async fn my_verify_password<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    password_plain_text: BBox<String, P1>,
    hash: BBox<String, P2>,
) -> Result<bool, ServiceError> {
    let res = execute_pcr((password_plain_text.clone(), hash.clone()), 
    PrivacyCriticalRegion::new(|(unboxed_password_plain, unboxed_hash): (String, String), _, _|{
        crate::crypto::verify_password(unboxed_password_plain, unboxed_hash)
    },
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();
    let dec_res = res.await;

    dec_res
}

// pub fn get_context() -> Context<TestContextData<()>> {
//     Context::empty()
// }