use std::future::Future;
use std::pin::Pin;

mod cloudflare_turnstile;

pub trait Verify {
    fn verify<'a>(&'a self, token: &'a str, ip: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}