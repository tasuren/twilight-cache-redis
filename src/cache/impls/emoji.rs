use twilight_model::id::{
    marker::{EmojiMarker, GuildMarker},
    Id,
};

use crate::{
    cache::{cmd, helper::*, Pipe, RedisKey, WithGuildId},
    traits::CacheStrategy,
    Error,
};

cmd::impl_set_wrapper_methods!(
    guild_emojis,
    key: {
        RedisKey::GuildEmojis: {
            guild_id: Id<GuildMarker>
        }
    },
    value: {
        emoji_id: Id<EmojiMarker>
    }
);
cmd::impl_str_wrapper_methods!(
    emoji,
    key: { emoji_id: Id<EmojiMarker> },
    value: WithGuildId<S::Emoji>
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_emoji(
        &mut self,
        guild_id: Id<GuildMarker>,
        addition_emoji_ids: impl Iterator<Item = Id<EmojiMarker>>,
    ) -> &mut Self {
        self.0.sadd(
            RedisKey::GuildEmojis { guild_id },
            addition_emoji_ids.collect_as_u64(),
        );
        self
    }

    pub(crate) fn remove_guild_emoji(
        &mut self,
        guild_id: Id<GuildMarker>,
        removal_emoji_id: Id<EmojiMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildEmojis { guild_id }, removal_emoji_id.get());
        self
    }

    pub(crate) fn remove_guild_emojis(
        &mut self,
        guild_id: Id<GuildMarker>,
        removal_emoji_ids: &[Id<EmojiMarker>],
    ) -> &mut Self {
        self.0.srem(
            RedisKey::GuildEmojis { guild_id },
            removal_emoji_ids.iter().copied().collect_as_u64(),
        );
        self
    }

    pub(crate) fn set_emojis(
        &mut self,
        emojis: impl Iterator<Item = (Id<EmojiMarker>, S::Emoji)>,
    ) -> Result<&mut Self, Error> {
        self.0.mset(&serialize_with_keys(emojis)?);
        Ok(self)
    }

    pub(crate) fn delete_emoji(&mut self, emoji_id: Id<EmojiMarker>) -> &mut Self {
        self.0.del(emoji_id.get());
        self
    }

    pub(crate) fn delete_emojis(
        &mut self,
        emoji_ids: impl Iterator<Item = Id<EmojiMarker>>,
    ) -> &mut Self {
        self.0.del(emoji_ids.map_redis_key());
        self
    }
}
