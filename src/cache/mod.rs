mod cmd;
pub mod helper;
mod impls;
mod key;
pub mod value;

pub(crate) use self::pipe::Pipe;
pub use self::{
    key::RedisKey,
    value::{FromCachedRedisValue, ToCachedRedisArg},
};
use crate::{Error, RedisCache};

pub mod pipe {
    use std::marker::PhantomData;

    use redis::{Pipeline, ToRedisArgs, Value};

    use crate::{CacheStrategy, Error};

    use super::FromCachedRedisValue;

    pub struct Pipe<S: CacheStrategy>(pub Pipeline, PhantomData<S>);

    impl<S: CacheStrategy> Pipe<S> {
        pub fn new() -> Self {
            Self(Pipeline::new(), PhantomData)
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
            T::from_redis_value(&value)
        }
    }
}

impl<S: crate::CacheStrategy> RedisCache<S> {
    pub(crate) async fn remove<T: FromCachedRedisValue>(
        &mut self,
        key: RedisKey,
    ) -> Result<Option<T>, Error> {
        let mut pipe = redis::Pipeline::new();
        let obj: (Option<redis::Value>,) = pipe
            .get(key)
            .del(key)
            .ignore()
            .query_async(&mut self.get_connection().await?)
            .await?;

        Ok(if let Some(data) = obj.0 {
            Some(T::from_redis_value(&data)?)
        } else {
            None
        })
    }
}
