use std::collections::{btree_map, BTreeMap, BinaryHeap};

pub trait CacheWeight {
    fn weight(&self) -> u64;
}

impl<T: CacheWeight> CacheWeight for Option<T> {
    fn weight(&self) -> u64 {
        match self {
            Some(x) => x.weight(),
            None => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheEntry<V> {
    weight: u64,
    frecency: ordered_float::OrderedFloat<f32>,
    value: V,
}

/// Each entry in the cache has a frecency, which is incremented by 1 on access and multiplied by
/// 0.8 on each evict so that entries age. Entries also have a weight, upon eviction entries will
/// be removed by order of least frecency until the max weight is respected. This cache only
/// removes entries on calls to `evict`, so the max weight may be exceeded until `evict` is called.
#[derive(Clone, Debug)]
pub struct FrecencyCache<K, V> {
    map: BTreeMap<K, CacheEntry<V>>,
    total_weight: u64,
}

impl<K: Ord, V> Default for FrecencyCache<K, V> {
    fn default() -> Self {
        FrecencyCache {
            map: BTreeMap::new(),
            total_weight: 0,
        }
    }
}

impl<K: Ord, V: CacheWeight> FrecencyCache<K, V> {
    pub fn new() -> Self {
        FrecencyCache {
            map: BTreeMap::new(),
            total_weight: 0,
        }
    }

    /// Updates and bumps freceny if already present.
    pub fn insert(&mut self, key: K, value: V) {
        let weight = value.weight();
        match self.map.entry(key) {
            btree_map::Entry::Vacant(entry) => {
                self.total_weight += weight;
                entry.insert(CacheEntry {
                    frecency: 1.0.into(),
                    weight,
                    value,
                });
            }
            btree_map::Entry::Occupied(mut entry) => {
                self.total_weight += weight - entry.weight;
                let entry = entry.get_mut();
                *entry.frecency += 1.0;
                entry.weight = weight;
                entry.value = value;
            }
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.map.get_mut(key).map(|e| {
            *e.frecency += 1.0;
            &e.value
        })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).map(|e| {
            self.total_weight -= e.weight;
            e.value
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn evict(&mut self, max_weight: u64) {
        use std::time::Instant;

        println!("EVICTING CACHE OF COUNT {}", self.map.len());

        if self.total_weight > max_weight as i64 {
            let start = Instant::now();

            // Build a priority queue of the cache keys by frecency while aging the entries.
            let mut queue: BinaryHeap<_> = self
                .0
                .iter_mut()
                .map(|(k, v)| {
                    *v.frecency *= 0.8;
                    (std::cmp::Reverse(v.frecency), v.weight, k.clone())
                })
                .collect();
            println!("TIME TO COLLECT {}", start.elapsed().as_millis());

            // We set the target below the maximum to delay the next evict since evict is O(n).
            let target_weight = 0.8 * self.max_weight as f32;
            while self.total_weight > target_weight {
                // Unwraps: If there were no entries left, `total_weight` would be 0.
                // If the key exists in the queue, it exists in the map.
                let key = queue.pop().unwrap().2;
                self.remove(&key).unwrap();
            }
            println!("TIME TO POP {}", start.elapsed().as_millis());
        }
    }
}

impl<K: Ord, V> IntoIterator for FrecencyCache<K, V> {
    type Item = (K, CacheEntry<V>);
    type IntoIter = btree_map::IntoIter<K, CacheEntry<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<K: Ord, V> Extend<(K, CacheEntry<V>)> for FrecencyCache<K, V> {
    fn extend<T: IntoIterator<Item = (K, CacheEntry<V>)>>(&mut self, iter: T) {
        self.map.extend(iter);
    }
}
