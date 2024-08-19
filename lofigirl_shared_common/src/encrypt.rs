use std::sync::LazyLock;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead}, aes::Aes256, Aes256Gcm, AesGcm, Key, KeyInit as _, Nonce
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize, Serializer};
use typenum::U12;

const ENCRYPTION_KEY_BYTES: &[u8; 32] = &[42; 32];
const ENCRYPTION_IV_BYTES: &[u8; 12] = &[42; 12];
static ENCRYPTION_KEY: LazyLock<Key<Aes256Gcm>> = LazyLock::new(|| (*ENCRYPTION_KEY_BYTES).into());
static AES_CIPHER: LazyLock<AesGcm<Aes256, U12>> = LazyLock::new(|| Aes256Gcm::new(&ENCRYPTION_KEY) );
static AES_IV: LazyLock<&'static GenericArray<u8, U12>> = LazyLock::new(|| Nonce::from_slice(ENCRYPTION_IV_BYTES));

#[derive(Debug, Clone)]
pub struct SecureString(pub String);

impl Serialize for SecureString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0.encrypt().map_err(|e| serde::ser::Error::custom(e))?)
    }
}

impl<'de> Deserialize<'de> for SecureString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(
            String::deserialize(deserializer)?
                .decrypt()
                .map_err(|e| serde::de::Error::custom(e))?,
        ))
    }
}

impl Into<String> for SecureString {
    fn into(self) -> String {
        self.0
    }
}

impl From<String> for SecureString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub trait AesEncryption {
    fn encrypt(&self) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn decrypt(&self) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl AesEncryption for String {
    fn encrypt(&self) -> anyhow::Result<Self> {
        let ciphertext = AES_CIPHER
            .encrypt(&AES_IV, self.as_bytes())
            .map_err(|e| anyhow::Error::msg(e))?;
        let plaintext = general_purpose::STANDARD.encode(&ciphertext);
        Ok(plaintext)
    }

    fn decrypt(&self) -> anyhow::Result<Self> {
        let ciphertext = general_purpose::STANDARD.decode(&self)?;
        let plaintext = AES_CIPHER
            .decrypt(&AES_IV, ciphertext.as_ref())
            .map_err(|e| anyhow::Error::msg(e))?;
        Ok(String::from_utf8(plaintext)?)
    }
}
