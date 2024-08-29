use twilight_model::gateway::payload::incoming::{
    IntegrationCreate, IntegrationDelete, IntegrationUpdate,
};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheStrategy, Error, RedisCache, UpdateCache,
};

impl<S: CacheStrategy> UpdateCache<S> for IntegrationCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            if let Some(guild_id) = self.guild_id {
                pipe.add_guild_integration(
                    guild_id,
                    self.id,
                    &S::GuildIntegration::from(self.0.clone()),
                )?;
            }
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for IntegrationDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            pipe.remove_guild_integration(self.guild_id, self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for IntegrationUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::INTEGRATION) {
            if let Some(guild_id) = self.guild_id {
                pipe.update_guild_integration(
                    guild_id,
                    self.id,
                    &S::GuildIntegration::from(self.0.clone()),
                )?;
            }
        }

        Ok(())
    }
}
