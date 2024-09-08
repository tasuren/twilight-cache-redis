use std::{collections::HashSet, iter::Map, marker::PhantomData};

use redis::Value;
use twilight_model::id::Id;

use super::{FromCachedRedisValue, RedisKey, ToBytes};
use crate::Error;

pub(crate) fn serialize_with_keys<I, T>(
    iter: impl Iterator<Item = (I, T)>,
) -> Result<Vec<(RedisKey, Vec<u8>)>, Error>
where
    I: Into<RedisKey>,
    T: ToBytes,
{
    iter.map(|(id, e)| Ok::<(RedisKey, Vec<u8>), Error>((id.into(), e.to_bytes()?)))
        .collect()
}

// RedisKey Helper:
pub trait MapRedisKey {
    fn map_redis_key(self) -> Vec<RedisKey>;
}

impl<I: Into<RedisKey>, T: Iterator<Item = I>> MapRedisKey for T {
    fn map_redis_key(self) -> Vec<RedisKey> {
        self.map(|id| id.into()).collect()
    }
}

// itoa helper:
pub trait IdIterHelper<I, A> {
    fn map_u64(self) -> Map<I, impl FnMut(A) -> u64>;
    fn collect_as_u64(self) -> Vec<u64>;
}

impl<I, M> IdIterHelper<I, Id<M>> for I
where
    I: Iterator<Item = Id<M>>,
{
    fn map_u64(self) -> Map<I, impl FnMut(Id<M>) -> u64> {
        self.map(|id| id.get())
    }

    fn collect_as_u64(self) -> Vec<u64> {
        self.map_u64().collect()
    }
}

impl<I, M, T> IdIterHelper<I, (Id<M>, T)> for I
where
    I: Iterator<Item = (Id<M>, T)>,
{
    fn map_u64(self) -> Map<I, impl FnMut((Id<M>, T)) -> u64> {
        self.map(|(id, _)| id.get())
    }

    fn collect_as_u64(self) -> Vec<u64> {
        self.map_u64().collect()
    }
}

// AsyncIter Helper:
pub struct AsyncIter<'a, V> {
    iter: redis::AsyncIter<'a, Value>,
    _resource: PhantomData<V>,
}

impl<'a, V: FromCachedRedisValue> AsyncIter<'a, V> {
    pub fn new(iter: redis::AsyncIter<'a, Value>) -> Self {
        Self {
            iter,
            _resource: PhantomData,
        }
    }

    pub async fn next_item(&mut self) -> Result<Option<V>, Error> {
        if let Some(v) = self.iter.next_item().await {
            Option::from_cached_redis_value(&v)
        } else {
            Ok(None)
        }
    }
}

impl<'a, V: Eq + std::hash::Hash + FromCachedRedisValue> AsyncIter<'a, V> {
    pub async fn collect_hash_set(&mut self) -> Result<HashSet<V>, Error> {
        let mut data = HashSet::new();
        while let Some(v) = self.next_item().await? {
            data.insert(v);
        }
        Ok(data)
    }
}
