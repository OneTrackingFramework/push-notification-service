extern crate biscuit;

use biscuit::jwa::{ContentEncryptionAlgorithm, EncryptionOptions, KeyManagementAlgorithm};
use biscuit::jwe::{Compact, RegisteredHeader};
use biscuit::jwk::JWK;
use biscuit::Empty;

pub struct JWEHelper {
    secret: Vec<u8>,
    cek_algorithm: KeyManagementAlgorithm,
    enc_algorithm: ContentEncryptionAlgorithm,
    nonce_counter: num::BigUint,
}

impl Default for JWEHelper {
    fn default() -> JWEHelper {
        JWEHelper {
            secret: Vec::new(),
            cek_algorithm: KeyManagementAlgorithm::DirectSymmetricKey,
            enc_algorithm: ContentEncryptionAlgorithm::A128GCM,
            nonce_counter: num::BigUint::from_bytes_le(&[0; 96 / 8]),
        }
    }
}

impl JWEHelper {
    pub fn new(secret: &str) -> JWEHelper {
        JWEHelper {
            secret: base64::decode(secret.to_owned()).unwrap(),
            ..Default::default()
        }
    }
    pub fn decrypt(&self, token: &str) -> Result<String, biscuit::errors::Error> {
        let key: JWK<Empty> = JWK::new_octet_key(self.secret.as_slice(), Default::default());
        let encrypted_jwe = Compact::<Vec<u8>, biscuit::Empty>::new_encrypted(token);

        let decrypted_jwe = encrypted_jwe.decrypt(&key, self.cek_algorithm, self.enc_algorithm)?;
        let decrypted_payload: &Vec<u8> = decrypted_jwe.payload()?;
        Ok(String::from_utf8((*decrypted_payload.to_owned()).to_vec())?)
    }

    #[allow(dead_code)]
    pub fn encrypt(&mut self, payload: &str) -> Result<String, biscuit::errors::Error> {
        let key: JWK<Empty> = JWK::new_octet_key(self.secret.as_slice(), Default::default());

        let jwe = Compact::new_decrypted(
            From::from(RegisteredHeader {
                cek_algorithm: self.cek_algorithm,
                enc_algorithm: self.enc_algorithm,
                ..Default::default()
            }),
            payload.as_bytes().to_vec(),
        );

        assert!(self.nonce_counter.bits() <= 96);
        let mut nonce_bytes = self.nonce_counter.to_bytes_le();
        nonce_bytes.resize(96 / 8, 0);
        let options = EncryptionOptions::AES_GCM { nonce: nonce_bytes };

        let encrypted_jwe = jwe.encrypt(&key, &options).unwrap();
        self.nonce_counter += 1u8;
        Ok(encrypted_jwe.encrypted()?.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn gen_key(key_size: usize) -> String {
        let mut rng = rand::thread_rng();

        let mut key = Vec::with_capacity(key_size);
        for _ in 0..key_size {
            key.push(rng.gen());
        }
        base64::encode(&key)
    }
    #[test]
    fn test_encryption_decryption() {
        let payload = String::from("Lorem ipsum dolor sit amet");
        let rand_key = gen_key(16);

        let mut jwe_helper = JWEHelper::new(&rand_key);
        let encrypted = jwe_helper.encrypt(&payload).unwrap();
        let decrypted = jwe_helper.decrypt(&encrypted).unwrap();
        assert_eq!(payload, decrypted);
    }
}
