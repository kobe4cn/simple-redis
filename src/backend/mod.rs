use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;

use crate::RespFrame;
#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug, Clone)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
}
impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl BackendInner{
//     pub fn new() -> Self {
//         BackendInner {
//             map: DashMap::new(),
//             hmap: DashMap::new(),
//         }
//     }
// }
impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
impl Backend {
    pub fn new() -> Self {
        Backend(Arc::new(BackendInner {
            map: DashMap::new(),
            hmap: DashMap::new(),
        }))
    }
    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }
    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }
    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }
    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        if !self.hmap.contains_key(&key) {
            self.hmap.insert(key.clone(), DashMap::new());
        }
        self.hmap.get_mut(&key).unwrap().insert(field, value);
    }
    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }
}
