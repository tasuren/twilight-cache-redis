use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    traits::CacheStrategy,
    Error,
};

cmd::impl_set_wrapper_methods!(
    user_guilds,
    key: {
        RedisKey::UserGuilds: {
            user_id: Id<UserMarker>
        }
    },
    value: { guild_id: Id<GuildMarker> }
);
cmd::impl_set_wrapper_methods!(
    guild_members,
    key: {
        RedisKey::GuildMembers: {
            guild_id: Id<GuildMarker>
        }
    },
    value: { member_id: Id<UserMarker> }
);
cmd::impl_global_set_wrapper_methods!(
    users,
    key: Users,
    value: { user_id: Id<UserMarker> }
);
cmd::impl_str_wrapper_methods!(
    user,
    key: { user_id: Id<UserMarker> },
    value: S::User
);
cmd::impl_str_wrapper_methods_with_two_id!(
    member,
    key: { guild_id: GuildMarker, user_id: UserMarker },
    value: S::Member
);

impl<S: CacheStrategy> Pipe<S> {
    /// This associates a user with a guild.
    pub(crate) fn add_user_guild(
        &mut self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::UserGuilds { user_id }, guild_id.get());
        self
    }

    pub(crate) fn remove_user_guild(
        &mut self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::UserGuilds { user_id }, guild_id.get());
        self
    }

    pub(crate) fn add_user(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0.sadd(RedisKey::Users, user_id.get());
        self
    }

    pub(crate) fn remove_user(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0
            .srem(RedisKey::Users, user_id.get())
            .del(RedisKey::from(user_id));
        self
    }

    pub(crate) fn set_user(
        &mut self,
        user_id: Id<UserMarker>,
        user: &S::User,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(user_id), user.to_bytes()?);
        Ok(self)
    }

    pub(crate) fn delete_user(&mut self, user_id: Id<UserMarker>) -> &mut Self {
        self.0.del(RedisKey::from(user_id));
        self
    }

    pub(crate) fn add_guild_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildMembers { guild_id }, user_id.get());
        self
    }

    pub(crate) fn remove_guild_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildMembers { guild_id }, user_id.get());
        self
    }

    pub(crate) fn set_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        member: &S::Member,
    ) -> Result<&mut Self, Error> {
        self.0
            .set(RedisKey::from((guild_id, user_id)), member.to_bytes()?);
        Ok(self)
    }

    pub(crate) fn delete_member(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0.del(RedisKey::from((guild_id, user_id)));
        self
    }
}
