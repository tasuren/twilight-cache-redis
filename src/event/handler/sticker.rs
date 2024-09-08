use twilight_model::{
    channel::message::Sticker,
    gateway::payload::incoming::GuildStickersUpdate,
    id::{
        marker::{GuildMarker, StickerMarker},
        Id,
    },
};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

pub fn cache_sticker<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    sticker: Sticker,
) -> Result<(), Error> {
    pipe.add_guild_sticker_id(guild_id, sticker.id)
        .set_sticker(guild_id, sticker.id, &S::Sticker::from(sticker))?;
    Ok(())
}

pub fn uncache_sticker<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    sticker_id: Id<StickerMarker>,
) {
    pipe.remove_guild_sticker_id(guild_id, sticker_id)
        .delete_sticker(sticker_id);
}

impl<S: CacheStrategy> UpdateCache<S> for GuildStickersUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::STICKER) {
            return Ok(());
        }

        let mut conn = cache.get_connection().await?;
        let mut guild_sticker_ids = cache
            .scan_guild_sticker_ids(&mut conn, self.guild_id)
            .await?;

        let additional = self.stickers.clone();

        while let Some(sticker_id) = guild_sticker_ids.next_item().await? {
            if !additional.iter().any(|s| s.id == sticker_id) {
                uncache_sticker(pipe, self.guild_id, sticker_id);
            }
        }

        for sticker in additional {
            cache_sticker(pipe, self.guild_id, sticker)?;
        }

        Ok(())
    }
}
