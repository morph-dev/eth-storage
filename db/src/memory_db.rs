use super::{Db, DbError};

use std::{cmp::Eq, collections::HashMap, hash::Hash};

#[derive(Default)]
pub struct MemoryDb<K, V> {
    data: HashMap<K, V>,
}

impl<K, V> MemoryDb<K, V> {
    pub fn new() -> Self {
        MemoryDb {
            data: HashMap::new(),
        }
    }
}

impl<K, V> Db<K, V> for MemoryDb<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn write(&mut self, key: &K, value: &V) -> Result<(), DbError> {
        self.data.insert(key.clone(), value.clone());
        Ok(())
    }

    fn read(&self, key: &K) -> Result<Option<V>, DbError> {
        let v = self.data.get(key).cloned();
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_ok, assert_ok_eq};

    use super::{Db, MemoryDb};

    #[test]
    fn test_read_missing() {
        let memory_db: MemoryDb<[u8; 4], [u16; 8]> = MemoryDb::new();
        let key = [1u8, 2, 3, 4];
        assert_ok_eq!(memory_db.read(&key), None);
    }

    #[test]
    fn test_write() {
        let mut memory_db: MemoryDb<[u8; 4], [u16; 8]> = MemoryDb::new();
        let key = [1u8, 2, 3, 4];
        let value = [1u16, 1, 2, 3, 5, 8, 13, 21];
        assert_ok!(memory_db.write(&key, &value));
        assert_ok_eq!(memory_db.read(&key), Some(value));
    }

    #[test]
    fn test_update() {
        let mut memory_db: MemoryDb<[u8; 4], [u16; 8]> = MemoryDb::new();
        let key = [1u8, 2, 3, 4];
        let value1 = [0u16, 1, 1, 2, 3, 5, 8, 13];
        let value2 = [1u16, 1, 2, 3, 5, 8, 13, 21];

        assert_ok!(memory_db.write(&key, &value1));
        assert_ok_eq!(memory_db.read(&key), Some(value1));

        assert_ok!(memory_db.write(&key, &value2));
        assert_ok_eq!(memory_db.read(&key), Some(value2));
    }
}
