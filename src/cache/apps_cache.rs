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

pub fn cache_path() -> Result<PathBuf, CacheError> {
    let xdg = BaseDirectories::with_prefix("latui");
    let path = xdg.place_cache_file("apps.json")?;
    Ok(path)
}

pub fn load_cache() -> Result<Vec<SearchableItem>, CacheError> {
    let path = cache_path()?;
    let data = fs::read_to_string(path)?;
    let cache: CachedApps = serde_json::from_str(&data)?;
    Ok(cache.apps)
}

pub fn save_cache(items: &[SearchableItem]) -> Result<(), CacheError> {
    let cache = CachedApps {
        apps: items.to_vec(),
    };
    let json = serde_json::to_string(&cache)?;
    let path = cache_path()?;
    fs::write(path, json)?;
    Ok(())
}
