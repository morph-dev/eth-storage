use errors::DbError;

pub mod errors;
pub mod memory_db;

pub trait Db<K, V> {
    fn write(&mut self, key: K, value: V) -> Result<(), DbError>;

    fn read(&self, key: &K) -> Result<Option<V>, DbError>;
}
