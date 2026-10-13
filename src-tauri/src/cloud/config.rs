use crate::error::{AppError, ErrorCode};
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudConfig {
    pub supabase_url: String,
    pub publishable_key: String,
    #[serde(default)]
    pub transport_enabled: bool,
}

impl CloudConfig {
    pub fn load() -> Result<Option<Self>, AppError> {
        if let (Ok(url), Ok(key)) = (
            env::var("M2_SUPABASE_URL"),
            env::var("M2_SUPABASE_PUBLISHABLE_KEY"),
        ) {
            let enabled = env::var("M2_SYNC_TRANSPORT_ENABLED")
                .ok()
                .is_some_and(|value| value.eq_ignore_ascii_case("true"));
            return Self::validate(Self {
                supabase_url: url,
                publishable_key: key,
                transport_enabled: enabled,
            })
            .map(Some);
        }
        let configured = env::var_os("M2_HEALTH_CLOUD_CONFIG").map(PathBuf::from);
        #[cfg(debug_assertions)]
        let configured = configured.or_else(|| {
            Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../config/cloud.local.json"))
        });
        let Some(path) = configured else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| {
            AppError::new(
                ErrorCode::Validation,
                "The local cloud configuration is invalid.",
            )
        })?;
        Self::validate(value).map(Some)
    }

    fn validate(mut value: Self) -> Result<Self, AppError> {
        value.supabase_url = value.supabase_url.trim_end_matches('/').to_owned();
        let url = reqwest::Url::parse(&value.supabase_url)
            .map_err(|_| AppError::new(ErrorCode::Validation, "The Supabase URL is invalid."))?;
        if url.scheme() != "https" || url.host_str().is_none() {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Cloud configuration requires an HTTPS Supabase URL.",
            ));
        }
        if !value.publishable_key.starts_with("sb_publishable_")
            || value.publishable_key.chars().any(char::is_whitespace)
        {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Cloud configuration requires a Supabase publishable key.",
            ));
        }
        Ok(value)
    }
}

pub struct CloudConfigState(pub Result<Option<CloudConfig>, AppError>);

impl Default for CloudConfigState {
    fn default() -> Self {
        Self(CloudConfig::load())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_https_and_publishable_keys() {
        assert!(CloudConfig::validate(CloudConfig {
            supabase_url: "https://example.supabase.co/".into(),
            publishable_key: "sb_publishable_example".into(),
            transport_enabled: false,
        })
        .is_ok());
        for (url, key) in [
            ("http://example.supabase.co", "sb_publishable_example"),
            ("https://example.supabase.co", "sb_secret_forbidden"),
            ("not-a-url", "sb_publishable_example"),
        ] {
            assert!(CloudConfig::validate(CloudConfig {
                supabase_url: url.into(),
                publishable_key: key.into(),
                transport_enabled: false,
            })
            .is_err());
        }
    }

    #[test]
    fn committed_example_keeps_transport_disabled_and_contains_no_real_project() {
        let source = include_str!("../../../config/cloud.example.json");
        let value: CloudConfig = serde_json::from_str(source).unwrap();
        assert!(!value.transport_enabled);
        assert!(value.supabase_url.contains("your-project-ref"));
        assert_eq!(value.publishable_key, "sb_publishable_replace_me");
    }
}
