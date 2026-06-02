use crate::error::ServiceError;
use xray_model::{AppState, GlobalSettings, InboundConfig, CertInfo, StoredInbound};

pub struct Storage {
    state_path: String,
}

impl Storage {
    pub fn new(settings: &GlobalSettings) -> Self {
        Self {
            state_path: format!("{}/state.json", settings.state_dir),
        }
    }

    pub fn load_or_default(&self) -> Result<AppState, ServiceError> {
        if !std::path::Path::new(&self.state_path).exists() {
            return Ok(AppState {
                settings: GlobalSettings::default(),
                stored_inbounds: vec![],
                stored_certs: vec![],
            });
        }

        let data = std::fs::read_to_string(&self.state_path)?;
        let state: AppState = serde_json::from_str(&data)
            .map_err(|e| ServiceError::Storage(format!("Failed to parse state: {}", e)))?;
        Ok(state)
    }

    pub fn save_state(
        &self,
        settings: &GlobalSettings,
        inbounds: &[InboundConfig],
        certs: &[CertInfo],
    ) -> Result<(), ServiceError> {
        let dir = std::path::Path::new(&self.state_path).parent().unwrap();
        std::fs::create_dir_all(dir)?;

        let state = AppState {
            settings: settings.clone(),
            stored_inbounds: inbounds.iter().map(StoredInbound::from).collect(),
            stored_certs: certs.to_vec(),
        };

        let json = serde_json::to_string_pretty(&state)?;
        std::fs::write(&self.state_path, json).map_err(ServiceError::from)
    }
}
