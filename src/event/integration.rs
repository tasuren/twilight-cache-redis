use twilight_model::{
    gateway::payload::incoming::{IntegrationCreate, IntegrationDelete, IntegrationUpdate},
    guild::GuildIntegration,
    id::{
        marker::{GuildMarker, IntegrationMarker},
        Id,
    },
};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheStrategy, Error, RedisCache, UpdateCache,
};

pub fn cache_integration<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    integration: GuildIntegration,
) -> Result<(), Error> {
    pipe.add_guild_integration(guild_id, integration.id)
        .set_integration(
            guild_id,
            integration.id,
            &S::GuildIntegration::from(integration),
        )?;

    Ok(())
}

pub fn uncache_integration<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    integration_id: Id<IntegrationMarker>,
) {
    pipe.remove_guild_integration(guild_id, integration_id)
        .delete_integration(guild_id, integration_id);
}

impl<S: CacheStrategy> UpdateCache<S> for IntegrationCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            if let Some(guild_id) = self.guild_id {
                cache_integration(pipe, guild_id, self.0.clone())?;
            }
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for IntegrationDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            uncache_integration(pipe, self.guild_id, self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for IntegrationUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            if let Some(guild_id) = self.guild_id {
                pipe.set_integration(
                    guild_id,
                    self.id,
                    &S::GuildIntegration::from(self.0.clone()),
                )?;
            }
        }

        Ok(())
    }
}
