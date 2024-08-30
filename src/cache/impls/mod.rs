use twilight_model::user::CurrentUser;

use crate::{CacheStrategy, Error};

use super::{Pipe, RedisKey, ToCachedRedisArg};

mod channel;
mod emoji;
mod guild;
mod integration;
mod message;
mod presence;
mod reaction;
mod user;

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn set_current_user(
        &mut self,
        current_user: CurrentUser,
    ) -> Result<&mut Self, Error> {
        self.0
            .set(RedisKey::CurrentUser, current_user.to_redis_arg()?);
        Ok(self)
    }
}
