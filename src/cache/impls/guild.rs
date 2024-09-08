use twilight_model::id::{marker::GuildMarker, Id};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    traits::CacheStrategy,
    Error,
};

cmd::impl_global_set_wrapper_methods!(
    unavailable_guild_id,
    key: UnavailableGuildId,
    value: { guild_id: Id<GuildMarker> }
);
cmd::impl_global_set_wrapper_methods!(
    guild_ids,
    key: GuildId,
    value: { guild_id: Id<GuildMarker> }
);
cmd::impl_str_wrapper_methods!(
    guild,
    key: { guild_id: Id<GuildMarker> },
    value: S::Guild
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_unavailable_guild_id(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.sadd(RedisKey::UnavailableGuildId, guild_id.get());
        self
    }

    pub(crate) fn remove_unavailable_guild_id(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.srem(RedisKey::UnavailableGuildId, guild_id.get());
        self
    }

    pub(crate) fn add_guild_id(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.sadd(RedisKey::GuildId, guild_id.get());
        self
    }

    pub(crate) fn set_guild(
        &mut self,
        guild_id: Id<GuildMarker>,
        guild: &S::Guild,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(guild_id), guild.to_bytes()?);

        Ok(self)
    }

    pub(crate) fn remove_guild_id(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.srem(RedisKey::GuildId, guild_id.get());
        self
    }

    pub(crate) fn delete_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.del(RedisKey::from(guild_id));

        self
    }
}
