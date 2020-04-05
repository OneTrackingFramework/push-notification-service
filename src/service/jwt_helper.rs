use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

pub struct JWTHelper {
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
   sub: String,
   company: String
}
impl JWTHelper {
    pub fn new(secret: &str) -> JWTHelper{
        JWTHelper { secret: secret.to_owned() }
    }
    pub fn decode(&self, token: &str) {
         let token_message = decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_ref()), &Validation::new(Algorithm::HS256));
    }
    pub fn encode(&self) -> String {
        "".to_owned()
    }
}