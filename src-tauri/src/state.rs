use anyhow::Result;
use netvan_collectors::CollectorEngine;
use netvan_core::db::Database;
use std::sync::Arc;

pub struct AppState {
    pub engine: Arc<CollectorEngine>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        netvan_core::paths::ensure_data_dir()?;
        let db = Database::open_default()?;
        let engine = CollectorEngine::new(db)?;
        Ok(Self { engine })
    }
}
