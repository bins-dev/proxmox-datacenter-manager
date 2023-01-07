use std::collections::HashMap;

use anyhow::Error;
use lazy_static::lazy_static;

use proxmox_schema::{ApiType, Schema};
use proxmox_section_config::{SectionConfig, SectionConfigData, SectionConfigPlugin};

use crate::{open_api_lockfile, replace_api_config, ApiLockGuard};
use pdm_api_types::{OpenIdRealmConfig, REALM_ID_SCHEMA};

lazy_static! {
    pub static ref CONFIG: SectionConfig = init();
}

fn init() -> SectionConfig {
    let obj_schema = match OpenIdRealmConfig::API_SCHEMA {
        Schema::Object(ref obj_schema) => obj_schema,
        _ => unreachable!(),
    };

    let plugin = SectionConfigPlugin::new(
        "openid".to_string(),
        Some(String::from("realm")),
        obj_schema,
    );
    let mut config = SectionConfig::new(&REALM_ID_SCHEMA);
    config.register_plugin(plugin);

    config
}

pub const DOMAINS_CFG_FILENAME: &str = "/etc/proxmox-datacenter-manager/auth/domains.cfg";
pub const DOMAINS_CFG_LOCKFILE: &str = "/etc/proxmox-datacenter-manager/auth/.domains.lock";

/// Get exclusive lock
pub fn lock_config() -> Result<ApiLockGuard, Error> {
    open_api_lockfile(DOMAINS_CFG_LOCKFILE, None, true)
}

pub fn config() -> Result<(SectionConfigData, [u8; 32]), Error> {
    let content =
        proxmox_sys::fs::file_read_optional_string(DOMAINS_CFG_FILENAME)?.unwrap_or_default();

    let digest = openssl::sha::sha256(content.as_bytes());
    let data = CONFIG.parse(DOMAINS_CFG_FILENAME, &content)?;
    Ok((data, digest))
}

pub fn save_config(config: &SectionConfigData) -> Result<(), Error> {
    let raw = CONFIG.write(DOMAINS_CFG_FILENAME, config)?;
    replace_api_config(DOMAINS_CFG_FILENAME, raw.as_bytes())
}

// shell completion helper
pub fn complete_realm_name(_arg: &str, _param: &HashMap<String, String>) -> Vec<String> {
    match config() {
        Ok((data, _digest)) => data.sections.iter().map(|(id, _)| id.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

pub fn complete_openid_realm_name(_arg: &str, _param: &HashMap<String, String>) -> Vec<String> {
    match config() {
        Ok((data, _digest)) => data
            .sections
            .iter()
            .filter_map(|(id, (t, _))| {
                if t == "openid" {
                    Some(id.to_string())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}
