use twilight_model::id::{marker::GuildMarker, Id};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    traits::CacheStrategy,
    Error,
};

cmd::impl_global_set_wrapper_methods!(
    unavailable_guilds,
    key: UnavailableGuilds,
    value: { guild_id: Id<GuildMarker> }
);
cmd::impl_global_set_wrapper_methods!(
    guilds,
    key: Guilds,
    value: { guild_id: Id<GuildMarker> }
);
cmd::impl_str_wrapper_methods!(
    guild,
    key: { guild_id: Id<GuildMarker> },
    value: S::Guild
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_unavailable_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.sadd(RedisKey::UnavailableGuilds, guild_id.get());
        self
    }

    pub(crate) fn remove_unavailable_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.srem(RedisKey::UnavailableGuilds, guild_id.get());
        self
    }

    pub(crate) fn add_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.sadd(RedisKey::Guilds, guild_id.get());
        self
    }

    pub(crate) fn remove_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.srem(RedisKey::Guilds, guild_id.get());
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

    pub(crate) fn delete_guild(&mut self, guild_id: Id<GuildMarker>) -> &mut Self {
        self.0.del(RedisKey::from(guild_id));

        self
    }
}
