use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::searchable_item::SearchableItem;

pub const APPS_CACHE_SCHEMA_VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct CachedApps {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub built_at_unix: u64,
    #[serde(default)]
    pub cache_key: String,
    pub apps: Vec<SearchableItem>,
}

fn default_schema_version() -> u32 {
    APPS_CACHE_SCHEMA_VERSION
}

use crate::error::CacheError;
use crate::core::utils::{current_timestamp, latui_xdg};
use tracing::{debug, info, instrument};

pub fn cache_path() -> Result<PathBuf, CacheError> {
    let xdg = latui_xdg();
    let path = xdg.place_cache_file("apps.json")?;
    Ok(path)
}

#[instrument]
pub fn load_cache() -> Result<CachedApps, CacheError> {
    let path = cache_path()?;
    debug!("Attempting to load index cache from {:?}", path);

    // Validation: Don't load massive cache files (> 10MB)
    let metadata = fs::metadata(&path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err(CacheError::Io(std::io::Error::other(
            "Cache file too large",
        )));
    }

    let data = fs::read_to_string(&path)?;
    let cache: CachedApps = serde_json::from_str(&data)?;
    info!(
        "Successfully loaded {} items from cache payload",
        cache.apps.len()
    );
    Ok(cache)
}

#[instrument(skip(items))]
pub fn save_cache(items: &[SearchableItem], cache_key: &str) -> Result<(), CacheError> {
    debug!("Serializing {} items to disk cache...", items.len());
    let cache = CachedApps {
        schema_version: APPS_CACHE_SCHEMA_VERSION,
        built_at_unix: current_timestamp(),
        cache_key: cache_key.to_string(),
        apps: items.to_vec(),
    };
    let json = serde_json::to_string(&cache)?;
    let path = cache_path()?;
    fs::write(&path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    debug!("Successfully flushed application state cache to {:?}", path);
    Ok(())
}
