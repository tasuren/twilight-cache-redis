use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    user::User,
};

use crate::{cache::Pipe, CacheStrategy, Error, RedisCache};

pub fn cache_user<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    user: User,
    guild_id: Option<Id<GuildMarker>>,
) -> Result<(), Error> {
    if let Some(guild_id) = guild_id {
        pipe.add_user_guild_id(user.id, guild_id);
    }

    pipe.add_user_id(user.id)
        .set_user(user.id, &S::User::from(user))?;

    Ok(())
}

pub fn uncached_user<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
) {
    pipe.delete_user(user_id)
        .remove_user_id(user_id)
        .remove_user_guild_id(user_id, guild_id);
}

pub async fn cache_users_only_already_cached<S: CacheStrategy>(
    cache: &mut RedisCache<S>,
    pipe: &mut Pipe<S>,
    users: &[(&User, Id<GuildMarker>)],
) -> Result<(), Error> {
    // Collect whether if the user is already contained in the cache.
    let is_contained: Vec<bool> = {
        let mut pipe = Pipe::<S>::new();

        for (user, _) in users {
            pipe.user_ids_contains(user.id);
        }

        pipe.query(&mut cache.get_connection().await?).await?
    };

    for (i, (user, guild_id)) in users.iter().copied().enumerate() {
        if !is_contained[i] {
            cache_user(pipe, user.clone(), Some(guild_id))?;
        }
    }

    Ok(())
}
