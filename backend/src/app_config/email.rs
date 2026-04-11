use lettre::message::Mailbox;
use serde::Deserialize;

use crate::{app_config::ConfigItem, error::fatal::FatalResult};

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    #[serde(flatten)]
    pub from: Mailbox,
    pub password: String,
    pub smtp_addr: String
}

impl ConfigItem for EmailConfig {
    type RuntimeConfig = EmailConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        Ok(self)
    }
}
