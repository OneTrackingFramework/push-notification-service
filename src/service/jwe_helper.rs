extern crate biscuit;

use biscuit::jwa::{ContentEncryptionAlgorithm, KeyManagementAlgorithm};
use biscuit::jwe::Compact;
use biscuit::jwk::JWK;
use biscuit::Empty;

pub struct JWEHelper {
    secret: Vec<u8>,
}

impl JWEHelper {
    pub fn new(secret: &str) -> JWEHelper {
        JWEHelper {
            secret: base64::decode(secret.to_owned()).unwrap(),
        }
    }
    pub fn decrypt(&self, token: &[u8]) -> Result<String, biscuit::errors::Error> {
        let key: JWK<Empty> = JWK::new_octet_key(self.secret.as_slice(), Default::default());
        let encrypted_jwe =
            Compact::<Vec<u8>, biscuit::Empty>::new_encrypted(std::str::from_utf8(token)?);

        let decrypted_jwe = encrypted_jwe.decrypt(
            &key,
            KeyManagementAlgorithm::DirectSymmetricKey,
            ContentEncryptionAlgorithm::A128GCM,
        )?;
        let decrypted_payload: &Vec<u8> = decrypted_jwe.payload()?;
        Ok(String::from_utf8((*decrypted_payload.to_owned()).to_vec())?)
    }
}
