use twilight_model::{
    gateway::payload::incoming::GuildEmojisUpdate,
    guild::Emoji,
    id::{marker::GuildMarker, Id},
};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheStrategy, Error, RedisCache, UpdateCache,
};

pub async fn cache_emojis<S: CacheStrategy>(
    cache: &mut RedisCache<S>,
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    mut incoming: Vec<Emoji>,
) -> Result<(), Error> {
    let mut conn = cache.get_connection().await?;

    let mut removal_emoji_ids = Vec::new();
    let additional_emojis = {
        let mut iter = cache.scan_guild_emoji_ids(&mut conn, guild_id).await?;
        while let Some(emoji_id) = iter.next_item().await {
            if let Some(i) = incoming.iter().position(|e| e.id == emoji_id) {
                incoming.remove(i);
            } else {
                removal_emoji_ids.push(emoji_id);
            }
        }

        incoming
    };

    if !additional_emojis.is_empty() {
        if cache.wants(ResourceType::USER) {
            // If user is set, add the user to the cache.
            for emoji in additional_emojis.iter() {
                if let Some(user) = &emoji.user {
                    pipe.add_user_id(user.id)
                        .add_user_guild_id(user.id, guild_id)
                        .set_user(user.id, &S::User::from(user.clone()))?;
                }
            }
        }

        pipe.add_guild_emoji_ids(guild_id, additional_emojis.iter().map(|e| e.id));
        pipe.set_emojis(
            additional_emojis
                .into_iter()
                .map(|e| (e.id, S::Emoji::from(e))),
        )?;
    }

    if !removal_emoji_ids.is_empty() {
        pipe.remove_guild_emoji_ids(guild_id, &removal_emoji_ids);
        pipe.delete_emojis(removal_emoji_ids.iter().copied());
    }

    Ok(())
}

impl<S: CacheStrategy> UpdateCache<S> for GuildEmojisUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::EMOJI) {
            return Ok(());
        }

        cache_emojis(cache, pipe, self.guild_id, self.emojis.clone()).await?;

        Ok(())
    }
}
