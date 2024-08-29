use twilight_model::{
    application::interaction::InteractionData, gateway::payload::incoming::InteractionCreate,
};

use crate::{
    cache::Pipe, config::ResourceType, event::user::cache_user, traits::CacheStrategy, Error,
    RedisCache, UpdateCache,
};

use super::member::cache_partial_member;

impl<S: CacheStrategy> UpdateCache<S> for InteractionCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::MEMBER) {
            // Cache interaction member
            if let (Some(member), Some(guild_id)) = (&self.member, self.guild_id) {
                if let Some(user) = &member.user {
                    cache_partial_member(pipe, guild_id, user.id, member.clone())?;
                }
            }
        }

        if cache.wants(ResourceType::USER) {
            if let Some(user) = &self.user {
                cache_user(pipe, user.clone(), self.guild_id)?;
            }
        }

        if let Some(InteractionData::ApplicationCommand(data)) = &self.data {
            if let Some(resolved) = &data.resolved {
                for user in resolved.users.values() {
                    if cache.wants(ResourceType::USER) {
                        cache_user(pipe, user.clone(), self.guild_id)?;
                    }

                    if !cache.wants(ResourceType::MEMBER) {
                        return Ok(());
                    }

                    if let Some(guild_id) = self.guild_id {
                        /*
                        // Implement this by `cache_interaction_member`
                        // after twilight 1.16 is coming.

                        if let Some(member) = &resolved.members.get(&user.id) {

                        } */

                        if cache.wants(ResourceType::ROLE) {
                            // TODO: Cache roles
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
