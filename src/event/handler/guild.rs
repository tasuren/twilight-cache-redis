use std::mem::take;

use twilight_model::{gateway::payload::incoming::GuildCreate, guild::Guild};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheStrategy, Error, RedisCache, UpdateCache,
};

async fn cache_guild<S: CacheStrategy>(
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
        pipe.remove_unavailable_guild_id(guild.id)
            .set_guild(guild.id, &S::Guild::from(guild.clone()))?;
    }

    Ok(())
}

impl<S: CacheStrategy> UpdateCache<S> for GuildCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        match self {
            g if g.0.unavailable => {
                if cache.wants(ResourceType::GUILD) {
                    pipe.add_unavailable_guild_id(g.id).delete_guild(g.id);
                }
                Ok(())
            }
            g => cache_guild(cache, pipe, g.0.clone()).await,
        }
    }
}
