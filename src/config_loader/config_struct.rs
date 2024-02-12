use std::fmt;
use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};

#[derive(Debug, Deserialize)]
pub struct InfluxConfig {
    pub url: String,
    pub token: String,
    pub org: String,
    pub bucket: String,
}

#[derive(Debug, Deserialize)]
pub struct OnedriveConfig {
    pub drive_type: String,     // "onedrive"
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum DriveConfig {
    Onedrive(OnedriveConfig)
}

impl<'de> Deserialize<'de> for DriveConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct DriveConfigVisitor;

        impl<'de> Visitor<'de> for DriveConfigVisitor {
            type Value = DriveConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct OnedriveConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DriveConfig, V::Error>
                where
                    V: MapAccess<'de>,
            {
                let mut drive_type = None;
                let mut refresh_token = None;
                let mut client_id = None;
                let mut client_secret = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "drive_type" => { drive_type = Some(map.next_value()?); },
                        "refresh_token" => { refresh_token = Some(map.next_value()?); },
                        "client_id" => { client_id = Some(map.next_value()?); },
                        "client_secret" => { client_secret = Some(map.next_value()?); },
                        _ => { return Err(de::Error::unknown_field(key, &["drive_type", "refresh_token", "client_id", "client_secret"])); }
                    }
                }

                let drive_type: String = drive_type.ok_or_else(|| de::Error::missing_field("drive_type"))?;
                let refresh_token: String = refresh_token.ok_or_else(|| de::Error::missing_field("refresh_token"))?;
                let client_id: String = client_id.ok_or_else(|| de::Error::missing_field("client_id"))?;
                let client_secret: String = client_secret.ok_or_else(|| de::Error::missing_field("client_secret"))?;

                if drive_type == "onedrive" {
                    Ok(DriveConfig::Onedrive(OnedriveConfig {
                        drive_type,
                        refresh_token,
                        client_id,
                        client_secret,
                    }))
                } else {
                    Err(de::Error::custom("drive_type not supported"))
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["drive_type", "refresh_token", "client_id", "client_secret"];
        deserializer.deserialize_struct("DriveConfig", FIELDS, DriveConfigVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub method: EncryptMethod,
    pub key: String
}

#[derive(Debug, Deserialize)]
pub enum EncryptMethod {
    RSA
}

#[derive(Debug, Deserialize)]
pub struct CacheSetting {
    pub refresh_interval: u64,  // in seconds, default to 600 seconds
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub influx: Option<InfluxConfig>,
    pub drives: Vec<DriveConfig>,
    pub encryption: Option<EncryptionConfig>,   // Optional, when not provided, encryption is disabled
    pub cache: Option<CacheSetting>,            // when not provided, cache will be set to default value
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_drive_config_onedrive() {
        let json = r#"
        {
            "drive_type": "onedrive",
            "refresh_token": "someToken",
            "client_id": "someId",
            "client_secret": "secret"
        }
        "#;

        let config: Result<DriveConfig, _> = serde_json::from_str(json);

        assert!(config.is_ok());

        if let Ok(DriveConfig::Onedrive(config)) = config {
            assert_eq!(config.drive_type, "onedrive");
            assert_eq!(config.refresh_token, "someToken");
            assert_eq!(config.client_id, "someId");
            assert_eq!(config.client_secret, "secret");
        } else {
            panic!("Expected Onedrive config");
        }
    }
}