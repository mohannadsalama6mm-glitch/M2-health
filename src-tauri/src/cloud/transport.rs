use super::{
    config::CloudConfig,
    models::{CloudLoginInput, CloudSessionStatus, PushReceipt, RemoteEvent, SyncEvent},
};
use crate::error::{AppError, ErrorCode};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::{sync::Mutex, time::SystemTime};

#[derive(Clone)]
struct CloudSession {
    local_user_id: String,
    access_token: String,
    refresh_token: String,
    subject_id: String,
    expires_at: i64,
}

pub struct CloudAuthState(Mutex<Option<CloudSession>>);

impl Default for CloudAuthState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

impl CloudAuthState {
    pub fn status_for(&self, local_user_id: &str) -> CloudSessionStatus {
        let session = self.0.lock().ok().and_then(|value| value.clone());
        let session = session.filter(|value| value.local_user_id == local_user_id);
        CloudSessionStatus {
            authenticated: session.is_some(),
            subject_id: session.as_ref().map(|value| value.subject_id.clone()),
            expires_at: session.as_ref().map(|value| value.expires_at),
        }
    }

    fn access_token(&self, config: &CloudConfig) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let session = self.0.lock().map_err(|_| {
            AppError::new(ErrorCode::Internal, "Cloud authentication is unavailable.")
        })?;
        if let Some(token) = session
            .as_ref()
            .filter(|value| value.expires_at > now + 30)
            .map(|value| value.access_token.clone())
        {
            return Ok(token);
        }
        let refresh_token = session
            .as_ref()
            .map(|value| value.refresh_token.clone())
            .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "Cloud sign-in is required."))?;
        drop(session);

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            refresh_token: String,
            expires_in: i64,
        }
        let response = Client::new()
            .post(format!(
                "{}/auth/v1/token?grant_type=refresh_token",
                config.supabase_url
            ))
            .header("apikey", &config.publishable_key)
            .json(&json!({"refresh_token":refresh_token}))
            .send()
            .map_err(network)?;
        if response.status() != StatusCode::OK {
            if let Ok(mut session) = self.0.lock() {
                *session = None;
            }
            return Err(remote_status(response.status()));
        }
        let body: RefreshResponse = response.json().map_err(network)?;
        let access_token = body.access_token.clone();
        let mut session = self.0.lock().map_err(|_| {
            AppError::new(ErrorCode::Internal, "Cloud authentication is unavailable.")
        })?;
        let current = session
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "Cloud sign-in is required."))?;
        current.access_token = body.access_token;
        current.refresh_token = body.refresh_token;
        current.expires_at = now + body.expires_in;
        Ok(access_token)
    }

    pub fn sign_out_for(&self, local_user_id: &str) {
        if let Ok(mut session) = self.0.lock() {
            if session
                .as_ref()
                .is_some_and(|value| value.local_user_id == local_user_id)
            {
                *session = None;
            }
        }
    }

    pub fn sign_in(
        &self,
        config: &CloudConfig,
        input: &CloudLoginInput,
        local_user_id: &str,
    ) -> Result<CloudSessionStatus, AppError> {
        if input.email.trim().is_empty() || input.password.is_empty() {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Cloud email and password are required.",
            ));
        }
        #[derive(Deserialize)]
        struct User {
            id: String,
        }
        #[derive(Deserialize)]
        struct Response {
            access_token: String,
            refresh_token: String,
            expires_in: i64,
            user: User,
        }
        let response = Client::new()
            .post(format!(
                "{}/auth/v1/token?grant_type=password",
                config.supabase_url
            ))
            .header("apikey", &config.publishable_key)
            .json(&json!({"email":input.email,"password":input.password}))
            .send()
            .map_err(network)?;
        if response.status() != StatusCode::OK {
            return Err(remote_status(response.status()));
        }
        let body: Response = response.json().map_err(network)?;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        *self.0.lock().map_err(|_| {
            AppError::new(ErrorCode::Internal, "Cloud authentication is unavailable.")
        })? = Some(CloudSession {
            local_user_id: local_user_id.to_owned(),
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            subject_id: body.user.id,
            expires_at: now + body.expires_in,
        });
        Ok(self.status_for(local_user_id))
    }
}

fn network(_: impl std::fmt::Display) -> AppError {
    AppError::new(
        ErrorCode::Internal,
        "The cloud service could not be reached. Local work remains available.",
    )
}

fn remote_status(status: StatusCode) -> AppError {
    match status {
        StatusCode::UNAUTHORIZED => AppError::new(
            ErrorCode::Unauthorized,
            "Cloud authentication is required or has expired.",
        ),
        StatusCode::FORBIDDEN => AppError::new(
            ErrorCode::Forbidden,
            "The cloud service denied this organization or device.",
        ),
        StatusCode::CONFLICT => AppError::new(
            ErrorCode::Conflict,
            "The cloud service rejected a conflicting event.",
        ),
        _ => AppError::new(
            ErrorCode::Internal,
            "The cloud service rejected the request. Local data was retained.",
        ),
    }
}

pub trait SyncTransport: Send + Sync {
    fn push(&self, events: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError>;
    fn pull(&self, after: i64, limit: usize) -> Result<Vec<RemoteEvent>, AppError>;
    fn acknowledge(&self, receipts: &[PushReceipt]) -> Result<(), AppError>;
}

pub struct SupabaseTransport<'a> {
    config: &'a CloudConfig,
    auth: &'a CloudAuthState,
    device_id: String,
    client: Client,
}

impl<'a> SupabaseTransport<'a> {
    pub fn new(config: &'a CloudConfig, auth: &'a CloudAuthState, device_id: String) -> Self {
        Self {
            config,
            auth,
            device_id,
            client: Client::new(),
        }
    }

    fn rpc<T: for<'de> Deserialize<'de>>(
        &self,
        name: &str,
        body: serde_json::Value,
    ) -> Result<T, AppError> {
        let token = self.auth.access_token(self.config)?;
        let response = self
            .client
            .post(format!("{}/rest/v1/rpc/{name}", self.config.supabase_url))
            .header("apikey", &self.config.publishable_key)
            .bearer_auth(token)
            .json(&body)
            .send()
            .map_err(network)?;
        if !response.status().is_success() {
            return Err(remote_status(response.status()));
        }
        response.json().map_err(network)
    }
}

impl SyncTransport for SupabaseTransport<'_> {
    fn push(&self, events: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError> {
        self.rpc(
            "push_sync_events",
            json!({"p_device_id":self.device_id,"p_events":events}),
        )
    }

    fn pull(&self, after: i64, limit: usize) -> Result<Vec<RemoteEvent>, AppError> {
        self.rpc(
            "pull_sync_events",
            json!({"p_device_id":self.device_id,"p_after_sequence":after,"p_limit":limit}),
        )
    }

    fn acknowledge(&self, receipts: &[PushReceipt]) -> Result<(), AppError> {
        let _: serde_json::Value = self.rpc(
            "ack_sync_events",
            json!({"p_device_id":self.device_id,"p_receipts":receipts}),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(owner: &str) -> CloudAuthState {
        CloudAuthState(Mutex::new(Some(CloudSession {
            local_user_id: owner.into(),
            access_token: "memory-only-access".into(),
            refresh_token: "memory-only-refresh".into(),
            subject_id: "cloud-subject".into(),
            expires_at: i64::MAX,
        })))
    }

    #[test]
    fn cloud_session_is_visible_and_revocable_only_by_its_local_owner() {
        let auth = state("local-owner");
        assert!(auth.status_for("local-owner").authenticated);
        assert!(!auth.status_for("other-local-user").authenticated);
        auth.sign_out_for("other-local-user");
        assert!(auth.status_for("local-owner").authenticated);
        auth.sign_out_for("local-owner");
        assert!(!auth.status_for("local-owner").authenticated);
    }
}
