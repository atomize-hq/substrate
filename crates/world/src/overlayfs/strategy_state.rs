use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use substrate_common::{WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe};

#[derive(Debug, Clone)]
pub struct WorldFsStrategyMeta {
    pub primary: WorldFsStrategy,
    pub final_strategy: WorldFsStrategy,
    pub fallback_reason: WorldFsStrategyFallbackReason,
    pub probe: Option<WorldFsStrategyProbe>,
}

static STRATEGY_BY_WORLD_ID: OnceLock<Mutex<HashMap<String, WorldFsStrategyMeta>>> =
    OnceLock::new();

fn state() -> &'static Mutex<HashMap<String, WorldFsStrategyMeta>> {
    STRATEGY_BY_WORLD_ID.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn set(world_id: &str, meta: WorldFsStrategyMeta) {
    let mut map = state().lock().expect("strategy state lock");
    map.insert(world_id.to_string(), meta);
}

pub fn get(world_id: &str) -> Option<WorldFsStrategyMeta> {
    let map = state().lock().expect("strategy state lock");
    map.get(world_id).cloned()
}

pub(crate) fn clear(world_id: &str) {
    let mut map = state().lock().expect("strategy state lock");
    map.remove(world_id);
}
