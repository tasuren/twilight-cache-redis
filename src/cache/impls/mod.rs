use redis::AsyncCommands;

use crate::{CacheStrategy, Connection, Error, RedisCache};

use super::{FromCachedRedisValue, Pipe, RedisKey, ToCachedRedisArg};

mod channel;
mod emoji;
mod guild;
mod integration;
mod message;
mod presence;
mod reaction;
mod user;

impl<S: CacheStrategy> RedisCache<S> {
    pub async fn get_current_user(
        &self,
        conn: &mut Connection<'_>,
    ) -> Result<S::CurrentUser, Error> {
        let raw: redis::Value = conn.get(RedisKey::CurrentUser).await?;
        S::CurrentUser::from_cached_redis_value(&raw)
    }
}

impl<S: CacheStrategy> Pipe<S> {
    pub fn get_current_user(&mut self) -> &mut Self {
        self.0.get(RedisKey::CurrentUser);
        self
    }

    pub(crate) fn set_current_user(
        &mut self,
        current_user: S::CurrentUser,
    ) -> Result<&mut Self, Error> {
        self.0
            .set(RedisKey::CurrentUser, current_user.to_redis_arg()?);
        Ok(self)
    }
}
