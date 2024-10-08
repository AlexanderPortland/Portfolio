use aes_gcm_siv::aead::Aead;
use aes_gcm_siv::KeyInit;
use argon2::{
    Argon2, PasswordHasher as ArgonPasswordHasher, PasswordVerifier as ArgonPasswordVerifier,
};
use async_compat::CompatExt;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as base64;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use rand::Rng;
use secrecy::ExposeSecret;
use std::iter;
use std::path::Path;
use std::str::FromStr;

use crate::error::ServiceError;

/// Foolproof random 12 char string
/// Only uppercase letters (except for O) and numbers (except for 0)
pub fn random_12_char_string() -> String {
    let random_chars_12: Vec<char> = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .map(char::from)
        .filter(is_usable_char)
        .take(12)
        .collect();
    
    random_chars_12
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join("")
}

/// Exclude O and 0, lowercase letters
fn is_usable_char(c: &char) -> bool {
    ('1'..='9').contains(c) ||
    ('A'..='N').contains(c) ||
    ('P'..'Z').contains(c) ||
    ['@', '#', '$', '%'].contains(c)
}

pub async fn hash_password(password_plain_text: String) -> Result<String, ServiceError> {
    let argon_config = Argon2::new(
        argon2::Algorithm::Argon2i,
        argon2::Version::V0x13,
        argon2::Params::new(6000, 3, 10, None)?,
    );

    let hash = tokio::task::spawn_blocking(move || {
        let password = password_plain_text.as_bytes();

        let salt_str = argon2::password_hash::SaltString::generate(rand::thread_rng());
        let salt = salt_str.as_salt();

        return argon_config
            .hash_password(password, salt)
            .map(|x| x.serialize().to_string());
    });

    let hash_string = hash.await??;

    Ok(hash_string)
}

pub async fn verify_password(
    password_plaint_text: String,
    hash: String,
) -> Result<bool, ServiceError> {
    let argon_config = Argon2::new(
        argon2::Algorithm::Argon2i,
        argon2::Version::V0x13,
        argon2::Params::new(6000, 3, 10, None)?,
    );

    let result: Result<bool, argon2::password_hash::Error> =
        tokio::task::spawn_blocking(move || {
            let parsed_hash = argon2::PasswordHash::new(&hash);
            match parsed_hash {
                Ok(parsed) => {
                    return Ok(argon_config
                        .verify_password(password_plaint_text.as_bytes(), &parsed)
                        .is_ok())
                }
                Err(error) => return Err(error),
            }
        })
        .await?;

    Ok(result?)
}

fn convert_key_aes256(key: &str) -> Vec<u8> {
    const REQUIRED_KEY_BYTES: usize = 32;
    //const REQUIRED_NONCE_BYTES: usize = 12;

    let key_len = key.as_bytes().len();

    if key_len >= REQUIRED_KEY_BYTES {
        return key.as_bytes().to_vec();
    }

    let multiplied_key: String = key.repeat((REQUIRED_KEY_BYTES / key_len) + 1);

    let key = multiplied_key.as_bytes().to_vec();

    key
}

pub async fn encrypt_password(
    password_plain_text: String,
    key: String,
) -> Result<String, ServiceError> {
    let hash = tokio::task::spawn_blocking(move || {
        let aes_key_nonce = convert_key_aes256(&key);

        // Nonce should be always unique, but for our use case it's fine
        // Also aes-gcm-siv is not vulnerable to nonce reuse
        let nonce = aes_gcm_siv::Nonce::from_slice(&aes_key_nonce[..12]);

        let cipher = aes_gcm_siv::Aes256GcmSiv::new_from_slice(&aes_key_nonce[..32]).unwrap();

        let res = cipher.encrypt(nonce, password_plain_text.as_bytes());
        res
    })
    .await??;

    Ok(base64.encode(hash))
}

pub async fn decrypt_password(
    password_cipher_text: String,
    key: String,
) -> Result<String, ServiceError> {
    let input = base64.decode(password_cipher_text)?;
    let plain = tokio::task::spawn_blocking(move || {
        let aes_key_nonce = convert_key_aes256(&key);

        let nonce = aes_gcm_siv::Nonce::from_slice(&aes_key_nonce[..12]);
        let cipher = aes_gcm_siv::Aes256GcmSiv::new_from_slice(&aes_key_nonce[..32]).unwrap();

        let res = cipher.decrypt(nonce, &*input);

        res
    })
    .await??;

    Ok(String::from_utf8(plain)?)
}

#[deprecated(note = "Too slow, use AES instead")]
pub async fn encrypt_password_age(
    password_plain_text: &str,
    key: &str,
) -> Result<String, ServiceError> {
    let encryptor = age::Encryptor::with_user_passphrase(age::secrecy::Secret::new(key.to_owned()));

    let mut encrypt_buffer = Vec::new();
    let mut encrypt_writer = encryptor.wrap_async_output(&mut encrypt_buffer).await?;

    encrypt_writer
        .write_all(password_plain_text.as_bytes())
        .await?;

    encrypt_writer.flush().await?;

    encrypt_writer.close().await?;

    Ok(base64.encode(encrypt_buffer))
}

#[deprecated(note = "Too slow, use AES instead")]
pub async fn decrypt_password_age(
    password_encrypted: &str,
    key: &str,
) -> Result<String, ServiceError> {
    let encrypted = base64.decode(password_encrypted)?;

    let decryptor = match age::Decryptor::new_async(&encrypted[..]).await? {
        age::Decryptor::Passphrase(d) => d,
        _ => unreachable!(),
    };

    let mut decrypt_buffer = Vec::new();
    let mut decrypt_writer =
        decryptor.decrypt_async(&age::secrecy::Secret::new(key.to_owned()), None)?;

    decrypt_writer.read_to_end(&mut decrypt_buffer).await?;

    Ok(String::from_utf8(decrypt_buffer)?)
}

pub fn create_identity() -> (String, String) {
    let identity = age::x25519::Identity::generate();

    // Public Key & Private Key
    (
        identity.to_public().to_string(),
        identity.to_string().expose_secret().to_string(),
    )
}

pub async fn encrypt_buffer_with_recipients(
    input_buffer: &[u8],
    recipients: &Vec<String>,
) -> Result<Vec<u8>, ServiceError> {
    let mut output_buffer = vec![];
    age_encrypt_with_recipients(input_buffer,
        &mut output_buffer,
        &recipients.iter().map(|s| s.as_str()).collect()
    ).await?;

    Ok(output_buffer)
}

async fn age_encrypt_with_recipients<W: tokio::io::AsyncWrite + Unpin>(
    input_buffer: &[u8],
    output_buffer: &mut W,
    recipients: &Vec<&str>,
) -> Result<(), ServiceError> {
    let public_keys = recipients
        .into_iter()
        .map(|recipient| {
            //TODO: No unwrap
            println!("recipient is {:?}", recipient.clone());
            Box::new(age::x25519::Recipient::from_str(recipient).unwrap()) as _
        })
        .collect();

    let encryptor_option = age::Encryptor::with_recipients(public_keys);

    if let Some(encryptor) = encryptor_option {
        let mut encrypt_writer = encryptor
            .wrap_async_output(output_buffer.compat_mut())
            .await?;

        encrypt_writer.write_all(input_buffer).await?;

        encrypt_writer.flush().await?;

        encrypt_writer.close().await?;

        return Ok(());
    } else {
        return Err(ServiceError::AgeNoRecipientsError);
    }
}

async fn age_decrypt_with_private_key<R: tokio::io::AsyncRead + Unpin>(
    input_buffer: R,
    output_buffer: &mut Vec<u8>,
    key: &str,
) -> Result<(), ServiceError> {
    let decryptor = match age::Decryptor::new_async(input_buffer.compat()).await? {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut decrypt_writer = decryptor.decrypt_async(iter::once(
        &age::x25519::Identity::from_str(key)
            .map_err(|e| ServiceError::AgeKeyError(e.to_string()))? as &dyn age::Identity,
    ))?;

    decrypt_writer.read_to_end(output_buffer).await?;

    Ok(())
}

pub async fn encrypt_password_with_recipients(
    password_plain_text: &str,
    recipients: &Vec<&str>,
) -> Result<String, ServiceError> {
    let mut encrypt_buffer = Vec::new();

    age_encrypt_with_recipients(
        password_plain_text.as_bytes(),
        &mut encrypt_buffer,
        recipients,
    )
    .await?;

    Ok(base64.encode(encrypt_buffer))
}

pub async fn decrypt_password_with_private_key(
    password_encrypted: &str,
    key: &str,
) -> Result<String, ServiceError> {
    let encrypted = base64.decode(password_encrypted)?;

    let mut decrypt_buffer = Vec::new();

    age_decrypt_with_private_key(encrypted.as_slice(), &mut decrypt_buffer, key).await?;

    let string = String::from_utf8(decrypt_buffer)?;
    Ok(string)
}

pub async fn encrypt_file_with_recipients<P: AsRef<Path>>(
    plain_file_path: P,
    cipher_file_path: P,
    recipients: Vec<&str>,
) -> Result<(), ServiceError> {
    let mut cipher_file = tokio::fs::File::create(cipher_file_path).await?;
    let mut plain_file = tokio::fs::File::open(plain_file_path).await?;

    let mut plain_file_contents = Vec::new();

    tokio::io::AsyncReadExt::read_to_end(&mut plain_file, &mut plain_file_contents).await?;

    drop(plain_file);

    age_encrypt_with_recipients(
        plain_file_contents.as_slice(),
        &mut cipher_file,
        &recipients,
    )
    .await?;

    tokio::io::AsyncWriteExt::shutdown(&mut cipher_file).await?;

    Ok(())
}

pub async fn decrypt_file_with_private_key<P: AsRef<Path>>(
    cipher_file_path: P,
    plain_file_path: P,
    key: &str,
) -> Result<(), ServiceError> {
    let cipher_file = tokio::fs::File::open(cipher_file_path).await?;
    let mut plain_file = tokio::fs::File::create(plain_file_path).await?;

    let mut plain_file_contents = Vec::new();

    age_decrypt_with_private_key(cipher_file, &mut plain_file_contents, key).await?;

    tokio::io::AsyncWriteExt::write_all(&mut plain_file, plain_file_contents.as_slice()).await?;

    Ok(())
}

pub async fn decrypt_file_with_private_key_as_buffer<P: AsRef<Path>>(
    cipher_file_path: P,
    key: &str,
) -> Result<Vec<u8>, ServiceError> {
    let cipher_file = tokio::fs::File::open(cipher_file_path).await?;

    let mut plain_file = Vec::new();

    age_decrypt_with_private_key(cipher_file, &mut plain_file, key).await?;

    Ok(plain_file)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as base64;

    #[test]
    fn test_random_12_char_string() {
        for _ in 0..1000 {
            let s = super::random_12_char_string();
            // Is 8 chars long
            assert_eq!(s.len(), 12);
            // Does not contain possibly confusing characters
            assert!(!s.contains('0'));
            assert!(!s.contains('O'));
        }
    }

    #[tokio::test]
    async fn test_hash_password() {
        const PASSWORD: &str = "test";
        let hash = super::hash_password(PASSWORD.to_string()).await.unwrap();

        assert!(hash.contains("$argon2"));
    }

    #[tokio::test]
    async fn test_verify_password() {
        const HASH: &str = "$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ";
        const PASSWORD: &str = "test";

        let result = super::verify_password(PASSWORD.to_string(), HASH.to_string())
            .await
            .unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn test_hash_and_verify_password() {
        const PASSWORD: &str = "test";

        let hash = super::hash_password(PASSWORD.to_string()).await.unwrap();

        let result = super::verify_password(PASSWORD.to_string(), hash)
            .await
            .unwrap();

        assert!(result);
    }

    #[test]
    fn test_convert_key_aes256() {
        let key_1 = super::convert_key_aes256("a");
        assert!(key_1.len() >= 32);

        let key_2 = super::convert_key_aes256(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(key_2.len() >= 32);

        let key_3 = super::convert_key_aes256(&super::random_12_char_string());
        assert!(key_3.len() >= 32);
    }

    #[tokio::test]
    async fn test_encrypt_password_is_valid_base64() {
        const PASSWORD: &str = "test";
        const KEY: &str = "testtesttesttesttesttest";

        let encrypted = super::encrypt_password(PASSWORD.to_string(), KEY.to_string())
            .await
            .unwrap();

        assert!(base64.decode(encrypted).is_ok());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_password() {
        const PASSWORD: &str = "test";
        const KEY: &str = "test";

        let encrypted = super::encrypt_password(PASSWORD.to_string(), KEY.to_string())
            .await
            .unwrap();

        let decrypted = super::decrypt_password(encrypted, KEY.to_string())
            .await
            .unwrap();

        assert_eq!(PASSWORD, decrypted);
    }

    #[tokio::test]
    async fn test_encrypt_password_age_is_valid_base64() {
        const PASSWORD: &str = "test";
        const KEY: &str = "testtesttesttesttesttest";

        #[allow(deprecated)]
        let encrypted = super::encrypt_password_age(PASSWORD, KEY).await.unwrap();

        assert!(base64.decode(encrypted).is_ok());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_age_password() {
        const PASSWORD: &str = "test";
        const KEY: &str = "test";

        #[allow(deprecated)]
        let encrypted = super::encrypt_password_age(PASSWORD, KEY).await.unwrap();
        #[allow(deprecated)]
        let decrypted = super::decrypt_password_age(&encrypted, KEY).await.unwrap();

        assert_eq!(PASSWORD, decrypted);
    }

    #[test]
    fn test_create_identity() {
        let identity = super::create_identity();

        assert!(identity.0.contains("age"));
        assert!(identity.1.contains("AGE-SECRET-KEY-"));
    }

    #[tokio::test]
    async fn test_encrypt_password_with_recipients_is_valid_base64() {
        const PASSWORD: &str = "test";
        const PUBLIC_KEY: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";

        let encrypted = super::encrypt_password_with_recipients(PASSWORD, &vec![PUBLIC_KEY])
            .await
            .unwrap();

        assert!(base64.decode(encrypted).is_ok());
    }

    #[tokio::test]
    async fn test_encrypt_password_with_recipients_multiple_is_valid_base64() {
        const PASSWORD: &str = "test";
        const PUBLIC_KEY_1: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";
        const PUBLIC_KEY_2: &str = "age1ygswsk38cq9r64um5klqxyvzemfdvx6qe5zed99pdexakwwhpatsgatgpw";

        let encrypted =
            super::encrypt_password_with_recipients(PASSWORD, &vec![PUBLIC_KEY_1, PUBLIC_KEY_2])
                .await
                .unwrap();

        assert!(base64.decode(encrypted).is_ok());
    }

    #[tokio::test]
    async fn test_decrypt_password_with_private_key() {
        const PASSWORD: &str = "test";
        //const PUBLIC_KEY: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";
        const PRIVATE_KEY: &str =
            "AGE-SECRET-KEY-1WPDHL2FLJ23T6RK5KCX8KS8DNLX0CGXMNZG0XNUAH4QP5C8ZZ46QGD3STV";
        const CIPHER: &str = "YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBVWUNCY0RielVCaThLbGlIR1NZa0p6MlNiS0x5L3B2Y3B2b21XZHNaZUVjClpsVTRvUGVVQVYzS205VTVVMDlXYjFHVE5ZZzJOSEpyN1ZyT0tocFpIbUUKLT4gPy1ncmVhc2UgLltXKT9MJyBLQGouLWcgfCBQSm12JQp3bDhRTDd0ZGZWbU9mQ2FYVU9Cb2FjM3AwR243OGJNCi0tLSBSSzRxV3E2d0VscERvM3VHVUhOL3dPaGVBRHE3WkZrdzYxYUgyQVl6elh3CiFQOr28YvbEAkx0YgFnIxwvPNjjYZV6THArcMPM8i5flnmKPw==";

        let decrypted = super::decrypt_password_with_private_key(CIPHER, PRIVATE_KEY)
            .await
            .unwrap();

        assert_eq!(PASSWORD, decrypted);
    }

    #[tokio::test]
    async fn test_decrypt_password_with_private_key_multiple() {
        const PASSWORD: &str = "test";
        // const PUBLIC_KEY_1: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";
        // const PUBLIC_KEY_2: &str = "age1ygswsk38cq9r64um5klqxyvzemfdvx6qe5zed99pdexakwwhpatsgatgpw";
        const PRIVATE_KEY_1: &str =
            "AGE-SECRET-KEY-1WPDHL2FLJ23T6RK5KCX8KS8DNLX0CGXMNZG0XNUAH4QP5C8ZZ46QGD3STV";
        const PRIVATE_KEY_2: &str =
            "AGE-SECRET-KEY-19RT6Z6TR0TE465EMJFDVXAFZ00YE65THLSS5LAY4W85L587DF95SPPDVND";

        const CIPHER: &str = "YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBBQ1BuSi9VMWIzeHg1TjQwMDNSUzlpZ0pGRWMxU2pFenVBekxGQTM0WGkwClkycytsNXNMbmVJTm5GT3VDRFBGQXE1ZFU5MzNzV0NXRWhmV1VGSjVNbU0KLT4gWDI1NTE5IHAvUjRLc3ROd2FkalZWTVIxRnBjaEluMXNtYWVScTVxdWxHY0x6ajZtUmMKYXkyNTExakZ0NWt5Vm85YUJSRlRmZTh4VEEyVEVrOFRyWDMxckNDVGkzOAotPiBbNVhfKS1ncmVhc2UgcysxIChlLTsKYU43T0lXUlUxZDFRVUpacXdJcm02Y3VzSjNMTVBtcy9pNm9yOEdETVplYjJrY1VsemRZU00rZ3NrSFZvUTBoSQovcEVrcmRmYlBPdzN3WWZTR0t1a1VFY0VTWXlIR1VPSUJRCi0tLSBYbmpxUHpVQzl5YnowdktIcTRjTklERXRDYVAxb0FmaWQwazgzRkp0U2pNCiAVlCPJ1+jroWQ7HBqjRUOcCBMyYvi9xIaklX2XDYPB2rd7Fw==";

        let decrypted_1 = super::decrypt_password_with_private_key(CIPHER, PRIVATE_KEY_1)
            .await
            .unwrap();

        assert_eq!(PASSWORD, decrypted_1);

        let decrypted_2 = super::decrypt_password_with_private_key(CIPHER, PRIVATE_KEY_2)
            .await
            .unwrap();

        assert_eq!(PASSWORD, decrypted_2);
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_file_with_recipients() {
        const PUBLIC_KEY: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";
        const PRIVATE_KEY: &str =
            "AGE-SECRET-KEY-1WPDHL2FLJ23T6RK5KCX8KS8DNLX0CGXMNZG0XNUAH4QP5C8ZZ46QGD3STV";

        const PASSWORD: &str = "test";

        let mut plain_file = async_tempfile::TempFile::new().await.unwrap();
        let mut encrypted_file = async_tempfile::TempFile::new().await.unwrap();

        tokio::io::AsyncWriteExt::write_all(&mut plain_file, PASSWORD.as_bytes())
            .await
            .unwrap();
        encrypted_file.sync_all().await.unwrap();

        assert_eq!(
            tokio::fs::read_to_string(&plain_file.file_path())
                .await
                .unwrap(),
            PASSWORD
        );

        super::encrypt_file_with_recipients(
            &plain_file.file_path(),
            &encrypted_file.file_path(),
            vec![PUBLIC_KEY],
        )
        .await
        .unwrap();
        encrypted_file.sync_all().await.unwrap();

        let mut buffer = [0; 21];

        tokio::io::AsyncReadExt::read(&mut encrypted_file, &mut buffer)
            .await
            .unwrap();

        assert_eq!(&buffer, b"age-encryption.org/v1");

        let decrypted_file = async_tempfile::TempFile::new().await.unwrap();

        super::decrypt_file_with_private_key(
            &encrypted_file.file_path(),
            &decrypted_file.file_path(),
            PRIVATE_KEY,
        )
        .await
        .unwrap();
        decrypted_file.sync_all().await.unwrap();

        assert_eq!(
            tokio::fs::read_to_string(&decrypted_file.file_path())
                .await
                .unwrap(),
            PASSWORD
        );
    }

    #[tokio::test]
    async fn test_decrypt_file_with_private_key_as_buffer() {
        const PUBLIC_KEY: &str = "age1t220v5c8ye0pjx99kw8nr57y7a5qlw4ke0wchjuxnr2gcvfzt3hq7fufz0";
        const PRIVATE_KEY: &str =
            "AGE-SECRET-KEY-1WPDHL2FLJ23T6RK5KCX8KS8DNLX0CGXMNZG0XNUAH4QP5C8ZZ46QGD3STV";

        const PASSWORD: &str = "test";

        let mut plain_file = async_tempfile::TempFile::new().await.unwrap();
        let encrypted_file = async_tempfile::TempFile::new().await.unwrap();

        tokio::io::AsyncWriteExt::write_all(&mut plain_file, PASSWORD.as_bytes())
            .await
            .unwrap();

        let plain_buffer = tokio::fs::read(&plain_file.file_path()).await.unwrap();

        assert_eq!(String::from_utf8(plain_buffer.clone()).unwrap(), PASSWORD);

        super::encrypt_file_with_recipients(
            &plain_file.file_path(),
            &encrypted_file.file_path(),
            vec![PUBLIC_KEY],
        )
        .await
        .unwrap();

        let decrypted_buffer =
            super::decrypt_file_with_private_key_as_buffer(encrypted_file.file_path(), PRIVATE_KEY)
                .await
                .unwrap();

        assert_eq!(plain_buffer.len(), decrypted_buffer.len());

        assert_eq!(
            String::from_utf8(decrypted_buffer.clone()).unwrap(),
            PASSWORD
        );
    }
}
