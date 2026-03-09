use std::fs;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::core::item::Item;

#[derive(Serialize, Deserialize)]
pub struct CachedApps {
    pub apps: Vec<Item>,
}

use xdg::BaseDirectories;

pub fn cache_path() -> PathBuf {
    let xdg = BaseDirectories::with_prefix("latui");
    xdg.place_cache_file("apps.json")
        .expect("Failed to create cache path")
}

pub fn load_cache() -> Option<Vec<Item>> {
    let path = cache_path();

    let data = fs::read_to_string(path).ok()?;

    let cache: CachedApps = serde_json::from_str(&data).ok()?;

    Some(cache.apps)
}

pub fn save_cache(items: &[Item]) {

    let cache = CachedApps {
        apps: items.to_vec(),
    };

    if let Ok(json) = serde_json::to_string(&cache) {

        let path = cache_path();

        let _ = fs::write(path, json);
    }
}
