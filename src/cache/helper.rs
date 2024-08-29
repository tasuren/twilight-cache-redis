use std::{collections::HashSet, iter::Map, marker::PhantomData};

use twilight_model::id::Id;

use crate::Error;
use super::{RedisKey, ToCachedRedisArg};

pub(crate) fn serialize_with_keys<I, T>(
    iter: impl Iterator<Item = (I, T)>,
) -> Result<Vec<(RedisKey, Vec<u8>)>, Error>
where
    I: Into<RedisKey>,
    T: ToCachedRedisArg,
{
    iter.map(|(id, e)| Ok::<(RedisKey, Vec<u8>), Error>((id.into(), e.to_redis_arg()?)))
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
pub struct IdAsyncIter<'a, M> {
    iter: redis::AsyncIter<'a, u64>,
    marker: PhantomData<Id<M>>,
}

impl<'a, M> IdAsyncIter<'a, M> {
    pub fn new(iter: redis::AsyncIter<'a, u64>) -> Self {
        Self {
            iter,
            marker: PhantomData,
        }
    }

    pub async fn next_item(&mut self) -> Option<Id<M>> {
        self.iter.next_item().await.map(|v| Id::new(v))
    }

    pub async fn to_hash_set(&mut self) -> HashSet<Id<M>> {
        let mut data = HashSet::new();
        while let Some(v) = self.next_item().await {
            data.insert(v);
        }
        data
    }
}
