use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::config_loader::config_struct::CaptchaConfig;

pub mod cloudflare_turnstile;
pub mod no_captcha;

pub trait Verify {
    fn verify<'a>(&'a self, token: &'a str, ip: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}

pub fn load_captcha(captcha_config: Option<CaptchaConfig>) -> Arc<dyn Verify> {
    match captcha_config {
        Some(config) => {
            if !config.enabled {
                Arc::new(no_captcha::NoCaptcha)
            } else {
                match config.service {
                    crate::config_loader::config_struct::SupportedCaptcha::CloudflareTurnstile => {
                        Arc::new(cloudflare_turnstile::CloudflareTurnstile::new(config.key))
                    }
                }
            }
        }
        None => {
            Arc::new(no_captcha::NoCaptcha)
        }
    }
}