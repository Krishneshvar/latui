use std::fs;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::core::searchable_item::SearchableItem;

#[derive(Serialize, Deserialize)]
pub struct CachedApps {
    pub apps: Vec<SearchableItem>,
}

use xdg::BaseDirectories;
use crate::error::CacheError;
use tracing::{info, debug, instrument};

pub fn cache_path() -> Result<PathBuf, CacheError> {
    let xdg = BaseDirectories::with_prefix("latui");
    let path = xdg.place_cache_file("apps.json")?;
    Ok(path)
}

#[instrument]
pub fn load_cache() -> Result<Vec<SearchableItem>, CacheError> {
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
    info!("Successfully loaded {} items from cache payload", cache.apps.len());
    Ok(cache.apps)
}

#[instrument(skip(items))]
pub fn save_cache(items: &[SearchableItem]) -> Result<(), CacheError> {
    debug!("Serializing {} items to disk cache...", items.len());
    let cache = CachedApps {
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
