use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    CacheStrategy, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_presences,
    key: {
        RedisKey::GuildPresences: {
            guild_id: Id<GuildMarker>
        }
    },
    value: { user_id: Id<UserMarker> }
);
cmd::impl_str_wrapper_methods_with_two_id!(
    presence,
    key: {
        RedisKey::Presence: {
            guild_id: Id<GuildMarker>,
            user_id: Id<UserMarker>
        }
    },
    value: S::Presence
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_presence(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildPresences { guild_id }, user_id.get());
        self
    }

    pub(crate) fn remove_guild_presence(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildPresences { guild_id }, user_id.get());
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

    pub(crate) fn delete_presence(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0.del(RedisKey::Presence { guild_id, user_id });
        self
    }
}
