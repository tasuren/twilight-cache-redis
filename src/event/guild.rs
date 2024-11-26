use std::mem::take;

use twilight_model::{
    gateway::payload::incoming::{GuildCreate, GuildDelete, GuildUpdate, UnavailableGuild},
    guild::Guild,
    id::{marker::GuildMarker, Id},
};

use crate::{
    cache::Pipe,
    config::ResourceType,
    traits::{CacheStrategy, CacheableGuild},
    Error, RedisCache, UpdateCache,
};

use super::channel::cache_channel;

pub async fn cache_guild<S: CacheStrategy>(
    cache: &mut RedisCache<S>,
    pipe: &mut Pipe<S>,
    mut guild: Guild,
) -> Result<(), Error> {
    if cache.wants(ResourceType::CHANNEL) {
        for mut channel in take(&mut guild.channels) {
            channel.guild_id = Some(guild.id);
            cache_channel(pipe, channel)?;
        }

        for mut thread in take(&mut guild.threads) {
            thread.guild_id = Some(guild.id);
            cache_channel(pipe, thread)?;
        }
    }

    if cache.wants(ResourceType::EMOJI) {
        super::emoji::cache_emojis(cache, pipe, guild.id, take(&mut guild.emojis)).await?;
    }

    if cache.wants(ResourceType::MEMBER) {
        for member in take(&mut guild.members) {
            super::member::cache_member(pipe, guild.id, member)?;
        }
    }

    if cache.wants(ResourceType::PRESENCE) {
        for presence in take(&mut guild.presences) {
            super::presence::cache_presence(pipe, presence)?;
        }
    }

    if cache.wants(ResourceType::ROLE) {
        for role in take(&mut guild.roles) {
            super::role::cache_role(pipe, guild.id, role)?;
        }
    }

    if cache.wants(ResourceType::STICKER) {
        for sticker in take(&mut guild.stickers) {
            super::sticker::cache_sticker(pipe, guild.id, sticker)?;
        }
    }

    if cache.wants(ResourceType::VOICE_STATE) {
        for voice_state in take(&mut guild.voice_states) {
            if let Some(channel_id) = voice_state.channel_id {
                super::voice_state::set_voice_state_cache(
                    pipe,
                    guild.id,
                    channel_id,
                    voice_state.user_id,
                    &S::VoiceState::from((guild.id, channel_id, voice_state)),
                )?;
            }
        }
    }

    if cache.wants(ResourceType::STAGE_INSTANCE) {
        for stage_instance in take(&mut guild.stage_instances) {
            super::stage_instance::cache_stage_instance(pipe, stage_instance)?;
        }
    }

    if cache.wants(ResourceType::GUILD) {
        pipe.remove_unavailable_guild(guild.id)
            .add_guild(guild.id)
            .set_guild(guild.id, &S::Guild::from(guild.clone()))?;
    }

    Ok(())
}

pub async fn uncache_guild<S: CacheStrategy>(
    cache: &mut RedisCache<S>,
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    unavailable: bool,
) -> Result<(), Error> {
    if cache.wants(ResourceType::GUILD) {
        if unavailable {
            if let Some(mut guild) = cache
                .get_guild(&mut cache.get_connection().await?, guild_id)
                .await?
            {
                guild.set_unavailable(true);
                pipe.set_guild(guild_id, &guild)?
                    .add_unavailable_guild(guild_id);
            }
        } else {
            pipe.remove_guild(guild_id);
        }
    }

    macro_rules! remove_ids {
        (
            cache.$scan_method:ident($($arg:expr),* $(,)?),
            $value_name:ident,
            $inner:tt
        ) => {
            let mut iter = cache.$scan_method($($arg),*).await?;

            while let Some($value_name) = iter.next_item().await? {
                $inner
            }
        };
    }

    if (cache.config.resource_type - ResourceType::GUILD).is_empty() {
        // If no other resource types are enabled, we shouldn't remove any other data.
        return Ok(());
    }

    let mut conn = cache.get_connection().await?;

    if cache.wants(ResourceType::CHANNEL) {
        remove_ids! {
            cache.scan_guild_channels(&mut conn, guild_id),
            id,
            {
                super::channel::uncache_channel(pipe, Some(guild_id), id);
            }
        };
    }

    if cache.wants(ResourceType::EMOJI) {
        remove_ids! {
            cache.scan_guild_emojis(&mut conn, guild_id),
            id,
            {
                super::emoji::uncache_emoji(pipe, guild_id, id);
            }
        }
    }

    if cache.wants(ResourceType::ROLE) {
        remove_ids! {
            cache.scan_guild_roles(&mut conn, guild_id),
            id,
            {
                super::role::uncache_role(pipe, guild_id, id);
            }
        }
    }

    if cache.wants(ResourceType::STICKER) {
        remove_ids! {
            cache.scan_guild_stickers(&mut conn, guild_id),
            id,
            {
                super::sticker::uncache_sticker(pipe, guild_id, id);
            }
        }
    }

    if cache.wants(ResourceType::VOICE_STATE) {
        remove_ids! {
            cache.scan_guild_members(&mut conn, guild_id),
            id,
            {
                super::member::uncache_member(pipe, guild_id, id);
            }
        }
    }

    if cache.wants(ResourceType::PRESENCE) {
        remove_ids! {
            cache.scan_guild_presences(&mut conn, guild_id),
            user_id,
            {
                super::presence::uncache_presence(pipe, guild_id, user_id);
            }
        }
    }

    Ok(())
}

impl<S: CacheStrategy> UpdateCache<S> for GuildCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        match self {
            g if g.0.unavailable => {
                if cache.wants(ResourceType::GUILD) {
                    pipe.add_unavailable_guild(g.id)
                        .remove_guild(g.id)
                        .delete_guild(g.id);
                }
                Ok(())
            }
            g => cache_guild(cache, pipe, g.0.clone()).await,
        }
    }
}

impl<S: CacheStrategy> UpdateCache<S> for GuildDelete {
    async fn update(
        &self,
        cache: &mut RedisCache<S>,
        pipe: &mut crate::cache::Pipe<S>,
    ) -> Result<(), Error> {
        uncache_guild(cache, pipe, self.id, false).await?;

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for UnavailableGuild {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        uncache_guild(cache, pipe, self.id, true).await?;

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
