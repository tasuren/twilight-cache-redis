use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    user::User,
};

use crate::{cache::Pipe, CacheStrategy, Error};

pub fn cache_user<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    user: User,
    guild_id: Option<Id<GuildMarker>>,
) -> Result<(), Error> {
    if let Some(guild_id) = guild_id {
        pipe.add_user_guild(user.id, guild_id);
    }

    pipe.add_user(user.id)
        .set_user(user.id, &S::User::from(user))?;

    Ok(())
}

pub fn uncached_user<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
) {
    pipe.delete_user(user_id)
        .remove_user(user_id)
        .remove_user_guild(user_id, guild_id);
}
