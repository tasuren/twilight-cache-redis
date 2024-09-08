mod cmd;
pub mod helper;
mod impls;
mod key;
pub mod value;

use std::fmt::Debug;

use redis::Value;
use twilight_model::id::{marker::GuildMarker, Id};

pub(crate) use self::pipe::Pipe;
pub use self::{
    key::RedisKey,
    value::{FromBytes, FromCachedRedisValue, ToBytes},
};
use crate::Error;

pub mod pipe {
    use std::marker::PhantomData;

    use redis::{Pipeline, ToRedisArgs, Value};

    use crate::{CacheStrategy, Error};

    use super::FromCachedRedisValue;

    pub struct Pipe<S: CacheStrategy>(pub Pipeline, PhantomData<S>);

    impl<S: CacheStrategy> Pipe<S> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn is_empty(&self) -> bool {
            self.0.cmd_iter().next().is_none()
        }

        pub fn atomic(&mut self) -> &mut Self {
            self.0.atomic();
            self
        }

        pub(crate) fn arg<T: ToRedisArgs>(&mut self, t: T) -> &mut Self {
            self.0.arg(t);
            self
        }

        pub async fn query<'a, T: FromCachedRedisValue>(
            &self,
            conn: &mut impl redis::aio::ConnectionLike,
        ) -> Result<T, Error> {
            let value: Value = self.0.query_async(conn).await?;
            T::from_cached_redis_value(&value)
        }
    }

    impl<S: CacheStrategy> Default for Pipe<S> {
        fn default() -> Self {
            Self(Pipeline::new(), PhantomData)
        }
    }
}

pub struct WithGuildId<T> {
    pub guild_id: Id<GuildMarker>,
    pub resource: T,
}

impl<T: ToBytes + FromBytes> WithGuildId<T> {
    pub fn new(guild_id: Id<GuildMarker>, resource: T) -> Result<Self, ()> {
        Ok(Self { guild_id, resource })
    }

    /// Make serialized `WithGuildId`.
    pub fn to_bytes(guild_id: Id<GuildMarker>, resource: &T) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&guild_id.get().to_be_bytes());
        bytes.extend_from_slice(&resource.to_bytes()?);

        Ok(bytes)
    }
}

impl<T: Clone> Clone for WithGuildId<T> {
    fn clone(&self) -> Self {
        Self {
            guild_id: self.guild_id,
            resource: self.resource.clone(),
        }
    }
}

impl<T: Debug> Debug for WithGuildId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WithGuildId")
            .field("guild_id", &self.guild_id)
            .field("resource", &self.resource)
            .finish()
    }
}

impl<T: FromBytes> FromCachedRedisValue for WithGuildId<T> {
    fn from_cached_redis_value(value: &redis::Value) -> Result<Self, Error> {
        if let Value::BulkString(bytes) = value {
            let mut guild_id: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
            guild_id.copy_from_slice(&bytes[..8]);
            let guild_id = Id::new(u64::from_be_bytes(guild_id));
            let resource = T::from_bytes(&bytes[8..])?;

            Ok(Self { guild_id, resource })
        } else {
            Err(Error::Parse {
                msg: "It is not `WithGuildId`.".to_owned(),
                response: format!("{value:?}"),
            })
        }
    }
}
