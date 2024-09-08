use redis::AsyncCommands;

use crate::{CacheStrategy, Connection, Error, RedisCache};

use super::{FromCachedRedisValue, Pipe, RedisKey, ToBytes};

mod channel;
mod emoji;
mod guild;
mod integration;
mod message;
mod presence;
mod role;
mod stage_instance;
mod sticker;
mod user;
mod voice_state;

impl<S: CacheStrategy> RedisCache<S> {
    pub async fn get_current_user(
        &self,
        conn: &mut Connection<'_>,
    ) -> Result<Option<S::CurrentUser>, Error> {
        let raw: redis::Value = conn.get(RedisKey::CurrentUser).await?;
        Option::from_cached_redis_value(&raw)
    }
}

impl<S: CacheStrategy> Pipe<S> {
    pub fn get_current_user(&mut self) -> &mut Self {
        self.0.get(RedisKey::CurrentUser);
        self
    }

    pub(crate) fn set_current_user(
        &mut self,
        current_user: &S::CurrentUser,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::CurrentUser, current_user.to_bytes()?);
        Ok(self)
    }
}
