use std::num::NonZeroUsize;

use lru::LruCache;

const NAME_MEMORY_SIZE: usize = 10;

pub struct NameMapper {
    all_list: Vec<String>,
    name_cache: LruCache<String, usize>,
}

impl NameMapper {
    pub fn new() -> Self {
        NameMapper {
            all_list: vec![],
            name_cache: LruCache::new(NonZeroUsize::new(NAME_MEMORY_SIZE).unwrap()),
        }
    }
    pub fn into_names(self) -> Vec<String> {
        self.all_list
    }
    /// Get mapping for given name.
    pub fn map_name(&mut self, name: &str) -> usize {
        if let Some(cached) = self.name_cache.get(name) {
            return *cached;
        }
        let new_idx = self.all_list.len();
        let name_key = name.to_owned();
        self.all_list.push(name_key.clone());
        self.name_cache.put(name_key, new_idx);
        new_idx
    }
}
