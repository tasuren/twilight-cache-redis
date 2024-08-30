use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

use crate::{
    cache::{Pipe, RedisKey, ToCachedRedisArg},
    CacheStrategy, Error,
};

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_role_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildRoleId { guild_id }, role_id.get());
        self
    }

    pub(crate) fn remove_guild_role_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildRoleId { guild_id }, role_id.get());
        self
    }

    pub(crate) fn set_role(
        &mut self,
        role_id: Id<RoleMarker>,
        role: &S::Role,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(role_id), role.to_redis_arg()?);
        Ok(self)
    }

    pub(crate) fn delete_role(&mut self, role_id: Id<RoleMarker>) -> &mut Self {
        self.0.del(RedisKey::from(role_id));
        self
    }
}
