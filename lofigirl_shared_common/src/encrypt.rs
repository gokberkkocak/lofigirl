use std::{fmt, sync::LazyLock};

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    aes::Aes256,
    AeadCore, Aes256Gcm, AesGcm, Key, KeyInit as _,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize, Serializer,
};
use typenum::U12;

const ENCRYPTION_KEY_BASE64: &[u8; 44] = include_bytes!("../../secrets/key.aes");

static AES_CIPHER: LazyLock<AesGcm<Aes256, U12>> = LazyLock::new(|| {
    let key_bytes = general_purpose::STANDARD
        .decode(ENCRYPTION_KEY_BASE64)
        .unwrap();
    let fixed_bytes: &[u8; 32] = key_bytes[..32].try_into().unwrap();
    let key: Key<Aes256Gcm> = (*fixed_bytes).into();
    Aes256Gcm::new(&key)
});

pub trait Aes256GCMEncryption {
    fn encrypt(&self) -> anyhow::Result<(Vec<u8>, GenericArray<u8, U12>)>
    where
        Self: Sized;
    fn decrypt(encrypted: Vec<u8>, nonce: GenericArray<u8, U12>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Aes256GCMEncryption for String {
    fn encrypt(&self) -> anyhow::Result<(Vec<u8>, GenericArray<u8, U12>)>{
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = AES_CIPHER
            .encrypt(&nonce, self.as_bytes())
            .map_err(anyhow::Error::msg)?;
        // let ciphertext_base64 = general_purpose::STANDARD.encode(&ciphertext);
        Ok((ciphertext, nonce))
    }

    fn decrypt(encrypted: Vec<u8>, nonce: GenericArray<u8, U12>) -> anyhow::Result<Self> {
        // let ciphertext = general_purpose::STANDARD.decode(self)?;
        let plaintext = AES_CIPHER
            .decrypt(&nonce, encrypted.as_ref())
            .map_err(anyhow::Error::msg)?;
        Ok(String::from_utf8(plaintext)?)
    }
}


#[derive(Debug, Clone)]
pub struct SecureString(String);

impl From<SecureString> for String {
    fn from(value: SecureString) -> Self {
        value.0
    }
}

impl From<String> for SecureString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Serialize for SecureString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (encrypted, nonce) = self.0.encrypt().map_err(serde::ser::Error::custom)?;
        let encrypted_base64 = general_purpose::STANDARD.encode(encrypted);
        let nonce_base64 = general_purpose::STANDARD.encode(nonce);
        let mut seq = serializer.serialize_map(Some(2))?;
        seq.serialize_entry("encrypted_base64", &encrypted_base64)?;
        seq.serialize_entry("nonce_base64", &nonce_base64)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for SecureString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CustomVisitor)
    }
}

struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = SecureString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map with keys 'encrypted_text' and 'nonce_base64'")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut encrypted_base64: Option<String> = None;
        let mut nonce_base64: Option<String> = None;

        while let Some(k) = map.next_key::<&str>()? {
            if k == "encrypted_base64" {
                encrypted_base64 = Some(map.next_value()?);
            } else if k == "nonce_base64" {
                nonce_base64 = Some(map.next_value()?);
            } else {
                return Err(serde::de::Error::custom(&format!("Invalid key: {}", k)));
            }
        }
        if let (Some(encrypted_base64), Some(nonce_base64)) = (encrypted_base64, nonce_base64) {
            let nonce_array = general_purpose::STANDARD
                .decode(nonce_base64)
                .map_err(serde::de::Error::custom)?;
            let nonce = GenericArray::clone_from_slice(&nonce_array);
            let encrypted = general_purpose::STANDARD
                .decode(encrypted_base64)
                .map_err(serde::de::Error::custom)?;
            let text = String::decrypt(encrypted, nonce).map_err(serde::de::Error::custom)?;
            Ok(SecureString(text))
        } else {
            Err(serde::de::Error::custom("Missing key(s)"))
        }
    }
}