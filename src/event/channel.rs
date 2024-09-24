use twilight_model::{
    channel::Channel,
    gateway::payload::incoming::{ChannelCreate, ChannelDelete, ChannelUpdate},
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheStrategy, Error, RedisCache, UpdateCache,
};

fn cache_channel_model<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    channel: Channel,
) -> Result<(), Error> {
    pipe.set_channel(channel.id, &S::Channel::from(channel))?;
    Ok(())
}

pub fn cache_channel<S: CacheStrategy>(pipe: &mut Pipe<S>, channel: Channel) -> Result<(), Error> {
    if let Some(guild_id) = channel.guild_id {
        pipe.add_guild_channel(guild_id, channel.id);
    }

    cache_channel_model(pipe, channel)
}

pub fn uncache_channel<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Option<Id<GuildMarker>>,
    channel_id: Id<ChannelMarker>,
) {
    if let Some(guild_id) = guild_id {
        pipe.remove_guild_channel(guild_id, channel_id);
    }

    pipe.delete_channel(channel_id);
}

impl<S: CacheStrategy> UpdateCache<S> for ChannelCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            cache_channel(pipe, self.0.clone())
        } else {
            Ok(())
        }
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ChannelUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            pipe.set_channel(self.id, &S::Channel::from(self.0.clone()))?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ChannelDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            uncache_channel(pipe, self.guild_id, self.id);
        }

        Ok(())
    }
}
