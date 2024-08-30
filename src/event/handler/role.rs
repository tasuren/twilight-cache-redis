use twilight_model::{
    gateway::payload::incoming::{RoleCreate, RoleDelete, RoleUpdate},
    guild::Role,
    id::{
        marker::{GuildMarker, RoleMarker},
        Id,
    },
};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

pub fn cache_role<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    role: Role,
) -> Result<(), Error> {
    pipe.add_guild_role_id(guild_id, role.id)
        .set_role(role.id, &S::Role::from(role))?;

    Ok(())
}

pub fn uncache_role<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
) {
    pipe.remove_guild_role_id(guild_id, role_id)
        .delete_role(role_id);
}

impl<S: CacheStrategy> UpdateCache<S> for RoleCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::ROLE) {
            cache_role(pipe, self.guild_id, self.role.clone())?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for RoleDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::ROLE) {
            uncache_role(pipe, self.guild_id, self.role_id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for RoleUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::ROLE) {
            cache_role(pipe, self.guild_id, self.role.clone())?;
        }

        Ok(())
    }
}
