use crate::service::captcha::Verify;

pub struct NoCaptcha;

#[async_trait::async_trait]
impl Verify for NoCaptcha {
    async fn verify<'a>(&'a self, _token: &'a str, _ip: &'a str) -> bool {
        true
    }
}