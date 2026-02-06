//! Sync client for communicating with tickit-sync server

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{SyncRecord, SyncRequest, SyncResponse, SyncStatus};
use crate::config::SyncConfig;

/// Client for syncing with a tickit-sync server
pub struct SyncClient {
    config: SyncConfig,
    device_id: Uuid,
    status: SyncStatus,
}

impl SyncClient {
    /// Create a new sync client
    pub fn new(config: SyncConfig) -> Self {
        // Generate or load persistent device ID
        let device_id = Self::get_or_create_device_id();

        Self {
            config,
            device_id,
            status: SyncStatus::default(),
        }
    }

    /// Check if sync is enabled and configured
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.config.server.is_some() && self.config.token.is_some()
    }

    /// Get current sync status
    pub fn status(&self) -> &SyncStatus {
        &self.status
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SyncConfig) {
        self.config = config;
    }

    /// Perform a sync operation
    pub fn sync(
        &mut self,
        local_changes: Vec<SyncRecord>,
        last_sync: Option<DateTime<Utc>>,
    ) -> Result<SyncResponse> {
        if !self.is_enabled() {
            anyhow::bail!("Sync is not enabled or not configured");
        }

        let server = self.config.server.as_ref().unwrap();
        let token = self.config.token.as_ref().unwrap();

        self.status.syncing = true;

        let request = SyncRequest {
            device_id: self.device_id,
            last_sync,
            changes: local_changes,
        };

        let result = self.do_sync(server, token, &request);

        self.status.syncing = false;

        match &result {
            Ok(response) => {
                self.status.last_sync = Some(response.server_time);
                self.status.last_error = None;
            }
            Err(e) => {
                self.status.last_error = Some(e.to_string());
            }
        }

        result
    }

    /// Perform the actual HTTP sync request
    fn do_sync(&self, server: &str, token: &str, request: &SyncRequest) -> Result<SyncResponse> {
        let url = format!("{}/api/v1/sync", server.trim_end_matches('/'));

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(30))
            .send_json(request)
            .context("Failed to connect to sync server")?;

        if response.status() != 200 {
            anyhow::bail!("Sync failed with status: {}", response.status());
        }

        response
            .into_json::<SyncResponse>()
            .context("Failed to parse sync response")
    }

    /// Get or create a persistent device ID
    fn get_or_create_device_id() -> Uuid {
        let path = dirs::config_dir()
            .map(|p| p.join("tickit").join(".device_id"))
            .unwrap_or_else(|| std::path::PathBuf::from(".tickit_device_id"));

        // Try to read existing
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(id) = Uuid::parse_str(content.trim())
        {
            return id;
        }

        // Create new
        let id = Uuid::new_v4();

        // Save it (ignore errors - will just create new next time)
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, id.to_string());

        id
    }

    /// Get the device ID
    pub fn device_id(&self) -> Uuid {
        self.device_id
    }
}
