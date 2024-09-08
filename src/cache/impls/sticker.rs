use twilight_model::id::{
    marker::{GuildMarker, StickerMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, WithGuildId},
    CacheStrategy, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_stickers,
    key: {
        RedisKey::GuildStickers: {
            guild_id: Id<GuildMarker>
        }
    },
    value: { sticker_id: Id<StickerMarker> }
);
cmd::impl_str_wrapper_methods!(
    sticker,
    key: { sticker_id: Id<StickerMarker> },
    value: WithGuildId<S::Sticker>
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_sticker(
        &mut self,
        guild_id: Id<GuildMarker>,
        sticker_id: Id<StickerMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildStickers { guild_id }, sticker_id.get());
        self
    }

    pub(crate) fn remove_guild_sticker(
        &mut self,
        guild_id: Id<GuildMarker>,
        sticker_id: Id<StickerMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildStickers { guild_id }, sticker_id.get());
        self
    }

    pub(crate) fn set_sticker(
        &mut self,
        guild_id: Id<GuildMarker>,
        sticker_id: Id<StickerMarker>,
        sticker: &S::Sticker,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::from(sticker_id),
            WithGuildId::to_bytes(guild_id, sticker)?,
        );
        Ok(self)
    }

    pub(crate) fn delete_sticker(&mut self, sticker_id: Id<StickerMarker>) -> &mut Self {
        self.0.del(RedisKey::from(sticker_id));
        self
    }
}
