use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::{
    cache::{Pipe, RedisKey, ToBytes},
    CacheStrategy, Error,
};

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_presence_user_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildPresenceUserId { guild_id }, user_id.get());
        self
    }

    pub(crate) fn set_presence(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        presence: &S::Presence,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::Presence { guild_id, user_id },
            presence.to_bytes()?,
        );
        Ok(self)
    }
}
