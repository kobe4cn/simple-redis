use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;

use crate::RespFrame;
#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug, Clone)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
    sis: DashMap<String, Vec<String>>,
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
            sis: DashMap::new(),
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
    pub fn sadd(&self, key: String, members: &Vec<String>) -> i64 {
        let mut set = self.sis.entry(key).or_default();
        let mut added = 0;
        for member in members {
            if !set.contains(member) {
                set.push(member.clone());
                added += 1;
            }
        }
        added
    }
    pub fn sismember(&self, key: String, member: String) -> i64 {
        let ret = self.sis.get(&key).map_or(false, |v| v.contains(&member));
        if ret {
            1
        } else {
            0
        }
    }
}
