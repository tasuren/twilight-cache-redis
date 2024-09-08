use std::mem::take;

use twilight_model::{
    gateway::payload::incoming::{GuildCreate, GuildUpdate, UnavailableGuild},
    guild::Guild,
    id::{marker::GuildMarker, Id},
};

use crate::{
    cache::Pipe,
    config::ResourceType,
    traits::{CacheStrategy, CacheableGuild},
    Error, RedisCache, UpdateCache,
};

pub async fn cache_guild<S: CacheStrategy>(
    cache: &mut RedisCache<S>,
    pipe: &mut Pipe<S>,
    mut guild: Guild,
) -> Result<(), Error> {
    if cache.wants(ResourceType::CHANNEL) {
        for mut channel in take(&mut guild.channels) {
            channel.guild_id = Some(guild.id);
            super::channel::cache_channel(pipe, channel)?;
        }
    }

    if cache.wants(ResourceType::EMOJI) {
        super::emoji::cache_emojis(cache, pipe, guild.id, take(&mut guild.emojis)).await?;
    }

    if cache.wants(ResourceType::GUILD) {
        pipe.remove_unavailable_guild(guild.id)
            .set_guild(guild.id, &S::Guild::from(guild.clone()))?;
    }

    Ok(())
}

pub fn uncache_guild<S: CacheStrategy>(pipe: &mut Pipe<S>, guild_id: Id<GuildMarker>) {
    pipe.add_unavailable_guild(guild_id).delete_guild(guild_id);

    // TODO: Do opposite of cache_guild
}

impl<S: CacheStrategy> UpdateCache<S> for GuildCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        match self {
            g if g.0.unavailable => {
                if cache.wants(ResourceType::GUILD) {
                    pipe.add_unavailable_guild(g.id).delete_guild(g.id);
                }
                Ok(())
            }
            g => cache_guild(cache, pipe, g.0.clone()).await,
        }
    }
}

impl<S: CacheStrategy> UpdateCache<S> for UnavailableGuild {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::GUILD) {
            uncache_guild(pipe, self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for GuildUpdate {
    async fn update(
        &self,
        cache: &mut RedisCache<S>,
        pipe: &mut crate::cache::Pipe<S>,
    ) -> Result<(), Error> {
        if cache.wants(ResourceType::GUILD) {
            if let Some(mut guild) = cache
                .get_guild(&mut cache.get_connection().await?, self.id)
                .await?
            {
                guild.update_with_guild_update(self);

                pipe.set_guild(self.id, &guild)?;
            }
        }

        Ok(())
    }
}
