use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, WithGuildId},
    CacheStrategy, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_roles,
    key: {
        RedisKey::GuildRoles: {
            guild_id: Id<GuildMarker>
        }
    },
    value: { role_id: Id<RoleMarker> }
);
cmd::impl_str_wrapper_methods!(
    guild_role_ids,
    key: { guild_id: Id<GuildMarker> },
    value: WithGuildId<S::Role>
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_role(
        &mut self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildRoles { guild_id }, role_id.get());
        self
    }

    pub(crate) fn remove_guild_role(
        &mut self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildRoles { guild_id }, role_id.get());
        self
    }

    pub(crate) fn set_role(
        &mut self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
        role: &S::Role,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::from(role_id),
            WithGuildId::to_bytes(guild_id, role)?,
        );
        Ok(self)
    }

    pub(crate) fn delete_role(&mut self, role_id: Id<RoleMarker>) -> &mut Self {
        self.0.del(RedisKey::from(role_id));
        self
    }
}
