use twilight_model::{gateway::{payload::incoming::PresenceUpdate, presence::Presence}, id::{marker::{GuildMarker, UserMarker}, Id}};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

pub fn cache_presence<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    presence: Presence,
) -> Result<(), Error> {
    pipe.add_guild_presence(presence.guild_id, presence.user.id())
        .set_presence(
            presence.guild_id,
            presence.user.id(),
            &S::Presence::from(presence),
        )?;

    Ok(())
}

pub fn uncache_presence<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
) {
    pipe.remove_guild_presence(guild_id, user_id)
        .delete_presence(guild_id, user_id);
}

impl<S: CacheStrategy> UpdateCache<S> for PresenceUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::PRESENCE) {
            cache_presence(pipe, self.0.clone())?;
        }

        Ok(())
    }
}
