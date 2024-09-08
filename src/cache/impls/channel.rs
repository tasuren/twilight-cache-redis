use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::{
    cache::{cmd, helper::*, RedisKey, ToBytes},
    CacheStrategy, Connection, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_channel,
    key: {
        RedisKey::GuildChannel: {
            guild_id: Id<GuildMarker>
        }
    },
    value: {
        channel_id: Id<ChannelMarker>
    }
);
cmd::impl_str_wrapper_methods!(
    channel,
    key: { channel_id: Id<ChannelMarker> },
    value: S::Channel
);

impl<S: CacheStrategy> crate::cache::Pipe<S> {
    pub(crate) fn add_guild_channel(
        &mut self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildChannel { guild_id }, channel_id.get());
        self
    }

    pub(crate) fn remove_guild_channel(
        &mut self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildChannel { guild_id }, channel_id.get());
        self
    }

    pub(crate) fn set_channel(
        &mut self,
        id: Id<ChannelMarker>,
        channel: &S::Channel,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(id), channel.to_bytes()?);
        Ok(self)
    }

    pub(crate) fn delete_channel(&mut self, id: Id<ChannelMarker>) -> &mut Self {
        self.0.del(RedisKey::from(id));
        self
    }
}
