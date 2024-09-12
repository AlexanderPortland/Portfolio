use std::path::Path;
use age::decryptor::RecipientsDecryptor;
use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::pcr::{execute_pcr, PrivacyCriticalRegion, Signature};
use alohomora::policy::{AnyPolicy, Policy, PolicyAnd};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora::testing::TestContextData;
use alohomora::unbox;
use argon2::Argon2;
use futures::channel::mpsc::Receiver;
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
    password_plain_text: String,
    key: BBox<String, P>
) -> Result<BBox<String, P>, ServiceError> {
    let enc_res = execute_pcr(key.clone(), 
        PrivacyCriticalRegion::new(|key, _, _|{
            crate::crypto::encrypt_password(password_plain_text, key)
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await;

    match enc_res {
        Ok(enc) => Ok(BBox::new(enc, key.policy().clone())),
        Err(e) => Err(e),
    }
}

pub async fn my_decrypt_password<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    ciphertext: BBox<String, P1>, key: BBox<String, P2>
) -> Result<BBox<String, AnyPolicy>, ServiceError> {
    let dec_res = execute_pcr((ciphertext.clone(), key.clone()), 
        PrivacyCriticalRegion::new(|(ciphertext, key), _, _|{
            crate::crypto::decrypt_password(ciphertext, key)
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap().await;

    match dec_res {
        Ok(dec) => {
            // very hacky strategy, but since we can't combine policies manually with only references, 
            // we do some fake combination in a ppr and then put our desired value inside
            let policy_box = execute_pure((ciphertext.clone(), key.clone(), dec), PrivacyPureRegion::new(|(c, k, dec): (String, String, String)|{
                let _ = c.contains(&k);
                dec
            })).unwrap();
            Ok(policy_box)
        },
        Err(e) => Err(e),
    }
}

pub async fn my_encrypt_password_with_recipients<P: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    password_plain_text: BBox<String, P>,
    recipients: &Vec<BBox<String, P2>>,
) -> Result<BBox<String, AnyPolicy>, ServiceError> {
    let (unboxed_password_plain_text, unboxed_recipients): (String, Vec<String>) = execute_pcr((password_plain_text.clone(), recipients.clone()), 
    PrivacyCriticalRegion::new(|(plaintext, recipients): (String, Vec<String>), _, _|{
        (plaintext, recipients)
    },
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();

    let recipients_ref = unboxed_recipients.iter().map(|s|s.as_str()).collect::<Vec<&str>>();

    let dec_res = crate::crypto::encrypt_password_with_recipients(&unboxed_password_plain_text, &recipients_ref).await;

    match dec_res {
        Ok(dec) => {
            // very hacky strategy, but since we can't combine policies manually with only references, 
            // we do some fake combination in a ppr and then put our desired value inside
            let policy_box = execute_pure((password_plain_text.clone(), recipients.clone(), dec), PrivacyPureRegion::new(|(c, k, dec): (_, _, String)|{
                dec
            })).unwrap();
            Ok(policy_box)
        },
        Err(e) => Err(e),
    }
}

pub async fn my_decrypt_password_with_private_key<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    password_encrypted: BBox<String, P1>,
    key: BBox<String, P2>,
) -> Result<BBox<String, AnyPolicy>, ServiceError> {
    let (unboxed_password_encrypted, unboxed_key): (String, String) = execute_pcr((password_encrypted.clone(), key.clone()), 
    PrivacyCriticalRegion::new(|(ciphertext, key): (String, String), _, _|{
        (ciphertext, key)
    },
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();
    let dec_res = crate::crypto::decrypt_password_with_private_key(&unboxed_password_encrypted, &unboxed_key).await;

    match dec_res {
        Ok(dec) => {
            // very hacky strategy, but since we can't combine policies manually with only references, 
            // we do some fake combination in a ppr and then put our desired value inside
            let policy_box = execute_pure((password_encrypted.clone(), key.clone(), dec), PrivacyPureRegion::new(|(c, k, dec): (String, String, String)|{
                let _ = c.contains(&k);
                dec
            })).unwrap();
            Ok(policy_box)
        },
        Err(e) => Err(e),
    }
}

pub async fn my_verify_password<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
    password_plain_text: BBox<String, P1>,
    hash: BBox<String, P2>,
) -> Result<bool, ServiceError> {
    let (unboxed_password_plain, unboxed_hash): (String, String) = execute_pcr((password_plain_text.clone(), hash.clone()), 
    PrivacyCriticalRegion::new(|(a, b): (String, String), _, _|{
        (a, b)
    },
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}, 
    Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();
    let dec_res = crate::crypto::verify_password(unboxed_password_plain, unboxed_hash).await;

    dec_res
}

// pub fn get_context() -> Context<TestContextData<()>> {
//     Context::empty()
// }