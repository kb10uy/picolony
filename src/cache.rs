use core::fmt::Debug;

use core::hash::Hash;
use heapless::FnvIndexMap;

pub struct SimpleCacheMap<K, V, const SIZE: usize> {
    index_map: FnvIndexMap<K, usize, SIZE>,
    data: [(K, V); SIZE],
    cursor: usize,
}

impl<K, V, const SIZE: usize> SimpleCacheMap<K, V, SIZE>
where
    K: Debug + Default + Copy + Eq + Hash,
    V: Default + Copy,
{
    /// Creates new instance.
    pub fn new() -> SimpleCacheMap<K, V, SIZE> {
        SimpleCacheMap {
            index_map: FnvIndexMap::new(),
            data: [(K::default(), V::default()); SIZE],
            cursor: 0,
        }
    }

    /// Gets cached value.
    pub fn get(&self, key: K) -> Option<&V> {
        self.index_map.get(&key).map(|&i| &self.data[i].1)
    }

    /// Puts new value to cache.
    pub fn put(&mut self, key: K, value: V) -> &V {
        let (reverse_key, _) = self.data[self.cursor];
        if self.index_map.contains_key(&reverse_key) {
            self.index_map.remove(&reverse_key);
        }

        // At least 1 space must be available
        self.data[self.cursor] = (key, value);
        let returning = &self.data[self.cursor].1;
        self.index_map
            .insert(key, self.cursor)
            .expect("No spece left");
        self.cursor += 1;

        returning
    }

    /// Queries key.
    /// If found, registered value will return.
    /// If not found, `generate_value()` will be called and it will register the value.
    pub fn get_or_else(
        &mut self,
        key: K,
        generate_value: impl FnOnce(K) -> Option<V>,
    ) -> Option<&V> {
        match self.index_map.get(&key) {
            Some(&index) => Some(&self.data[index].1),
            None => {
                let new_value = generate_value(key)?;
                Some(self.put(key, new_value))
            }
        }
    }
}
