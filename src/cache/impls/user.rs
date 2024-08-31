use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToCachedRedisArg},
    traits::CacheStrategy,
    Error,
};

cmd::impl_set_wrapper_methods!(
    user_guild_ids,
    UserGuildId,
    user_id,
    guild_id,
    UserMarker,
    GuildMarker
);
cmd::impl_set_wrapper_methods!(
    guild_member_ids,
    GuildMemberId,
    guild_id,
    member_id,
    GuildMarker,
    UserMarker
);
cmd::impl_global_set_wrapper_methods!(user_ids, UserId, user_id, UserMarker);
cmd::impl_str_wrapper_methods!(
    user,
    key: { user_id: Id<UserMarker> },
    value: User
);
cmd::impl_str_wrapper_methods_with_two_id!(
    member,
    guild_id,
    user_id,
    GuildMarker,
    UserMarker,
    Member
);

impl<S: CacheStrategy> Pipe<S> {
    /// This associates a user with a guild.
    pub(crate) fn add_user_guild_id(
        &mut self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::UserGuildId { user_id }, guild_id.get());
        self
    }

    pub(crate) fn remove_user_guild_id(
        &mut self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::UserGuildId { user_id }, guild_id.get());
        self
    }

    pub(crate) fn set_user(
        &mut self,
        user_id: Id<UserMarker>,
        user: &S::User,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(user_id), user.to_redis_arg()?);
        Ok(self)
    }

    pub(crate) fn add_user_id(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0.sadd(RedisKey::UserId, user_id.get());
        self
    }

    pub(crate) fn delete_user(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0.del(RedisKey::from(user_id));
        self
    }

    pub(crate) fn remove_user_id(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0
            .srem(RedisKey::UserId, user_id.get())
            .del(RedisKey::from(user_id));
        self
    }

    pub(crate) fn set_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        member: &S::Member,
    ) -> Result<&mut Self, Error> {
        self.0
            .set(RedisKey::from((guild_id, user_id)), member.to_redis_arg()?);
        Ok(self)
    }

    pub(crate) fn add_guild_member_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildMemberId { guild_id }, user_id.get());
        self
    }

    pub(crate) fn delete_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0.del(RedisKey::from((guild_id, user_id)));
        self
    }

    pub(crate) fn remove_guild_member_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildMemberId { guild_id }, user_id.get());
        self
    }
}
