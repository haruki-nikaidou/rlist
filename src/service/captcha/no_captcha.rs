use std::future::Future;
use std::pin::Pin;
use crate::service::captcha::Verify;

pub struct NoCaptcha;
impl Verify for NoCaptcha {
    fn verify<'a>(&'a self, _token: &'a str, _ip: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true })
    }
}