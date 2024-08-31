use redis::AsyncCommands;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::{
    cache::{cmd, helper::*, RedisKey, ToCachedRedisArg},
    CacheStrategy, Connection, Error, RedisCache,
};

cmd::impl_set_wrapper_methods!(
    guild_channel_ids,
    GuildChannelId,
    guild_id,
    channel_id,
    GuildMarker,
    ChannelMarker
);
cmd::impl_str_wrapper_methods!(
    channel,
    key: { channel_id: Id<ChannelMarker> },
    value: Channel
);

impl<S: CacheStrategy> RedisCache<S> {
    pub async fn get_guild_channel_ids<'a, 'stmt>(
        &self,
        conn: &'stmt mut Connection<'a>,
        guild_id: Id<GuildMarker>,
    ) -> Result<IdAsyncIter<'stmt, Id<GuildMarker>>, Error> {
        let iter = conn.sscan(RedisKey::GuildChannelId { guild_id }).await?;
        Ok(IdAsyncIter::new(iter))
    }
}

impl<S: CacheStrategy> crate::cache::Pipe<S> {
    pub(crate) fn add_guild_channel_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildChannelId { guild_id }, channel_id.get());
        self
    }

    pub(crate) fn remove_guild_channel_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildChannelId { guild_id }, channel_id.get());
        self
    }

    pub(crate) fn set_channel(
        &mut self,
        id: Id<ChannelMarker>,
        channel: &S::Channel,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(id), channel.to_redis_arg()?);
        Ok(self)
    }

    pub(crate) fn delete_channel(&mut self, id: Id<ChannelMarker>) -> &mut Self {
        self.0.del(RedisKey::from(id));
        self
    }
}
