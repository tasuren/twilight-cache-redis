use twilight_model::{
    gateway::payload::incoming::{MemberAdd, MemberChunk, MemberRemove, MemberUpdate},
    guild::{Member, PartialMember},
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::{
    cache::Pipe,
    config::ResourceType,
    event::user,
    traits::{CacheStrategy, CacheableGuild, CacheableMember},
    Error, RedisCache, UpdateCache,
};

fn cache_member_impl<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    member: &S::Member,
) -> Result<(), Error> {
    pipe.set_member(guild_id, user_id, member)?
        .add_guild_member(guild_id, user_id);

    Ok(())
}

pub fn cache_partial_member<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    member: PartialMember,
) -> Result<(), Error> {
    cache_member_impl(pipe, guild_id, user_id, &S::Member::from((user_id, member)))
}

/*
pub fn cache_interaction_member<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    member: InteractionMember,
) -> Result<(), Error> {
    todo!("Implement this function after twilight 1.16 is coming.")
} */

pub fn cache_member<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    member: Member,
) -> Result<(), Error> {
    pipe.add_guild_member(guild_id, member.user.id).set_member(
        guild_id,
        member.user.id,
        &S::Member::from(member),
    )?;
    Ok(())
}

pub fn uncache_member<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
) {
    pipe.delete_member(guild_id, user_id)
        .remove_guild_member(guild_id, user_id);
}

impl<S: CacheStrategy> UpdateCache<S> for MemberAdd {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::GUILD) {
            if let Some(mut guild) = cache
                .get_guild(&mut cache.get_connection().await?, self.guild_id)
                .await?
            {
                guild.increase_member_count(1);
                pipe.set_guild(self.guild_id, &guild)?;
            };
        }

        if cache.wants(ResourceType::USER) {
            user::cache_user(pipe, self.user.clone(), Some(self.guild_id))?;
        }

        if cache.wants(ResourceType::MEMBER) {
            cache_member(pipe, self.guild_id, self.member.clone())?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for MemberChunk {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if self.members.is_empty() {
            return Ok(());
        }

        if cache.wants(ResourceType::USER) {
            for member in self.members.iter() {
                user::cache_user(pipe, member.user.clone(), Some(self.guild_id))?;
            }
        }

        if cache.wants(ResourceType::MEMBER) {
            for member in self.members.iter() {
                cache_member(pipe, self.guild_id, member.clone())?;
            }
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for MemberRemove {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::GUILD) {
            if let Some(mut guild) = cache
                .get_guild(&mut cache.get_connection().await?, self.guild_id)
                .await?
            {
                guild.decrease_member_count(1);
                pipe.set_guild(self.guild_id, &guild)?;
            };
        }

        if cache.wants(ResourceType::USER) {
            user::uncache_user(pipe, self.user.id, self.guild_id);
        }

        if cache.wants(ResourceType::MEMBER) {
            uncache_member(pipe, self.guild_id, self.user.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for MemberUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::MEMBER) {
            if let Some(mut member) = cache
                .get_member(
                    &mut cache.get_connection().await?,
                    self.guild_id,
                    self.user.id,
                )
                .await?
            {
                member.update_with_member_update(self);
                pipe.set_member(self.guild_id, self.user.id, &member)?;
            };
        }

        if cache.wants(ResourceType::USER) {
            pipe.set_user(self.user.id, &S::User::from(self.user.clone()))?;
        }

        Ok(())
    }
}
