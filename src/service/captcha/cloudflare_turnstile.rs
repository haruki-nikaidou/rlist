use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::service::captcha::Verify;

const API_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Deserialize)]
struct Response {
    success: bool,
}

pub struct CloudflareTurnstile {
    secret: String,
}

impl CloudflareTurnstile {
    pub fn new(secret: String) -> Self {
        CloudflareTurnstile {
            secret
        }
    }
}

#[derive(Serialize)]
struct ApiForm {
    secret: String,
    response: String,
    remoteip: String,
    idempotency_key: String,
}

#[async_trait::async_trait]
impl Verify for CloudflareTurnstile {
    async fn verify<'a>(&'a self, token: &'a str, ip: &'a str) -> bool {
        let form = ApiForm {
            secret: self.secret.clone(),
            response: token.to_string(),
            remoteip: ip.to_string(),
            idempotency_key: Uuid::new_v4().to_string(),
        };
        let client = reqwest::Client::new();
        let response = client.post(API_URL)
            .form(&form)
            .send()
            .await;
        match response {
            Ok(response) => {
                let response = response.json::<Response>().await;
                match response {
                    Ok(response) => {
                        response.success
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
            Err(_) => {
                return false;
            }
        }
    }
}
