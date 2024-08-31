use twilight_model::id::{
    marker::{GuildMarker, IntegrationMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToCachedRedisArg},
    traits::CacheStrategy,
    Connection, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_integration_ids,
    key: {
        RedisKey::GuildIntegrationId: {
            guild_id: Id<GuildMarker>
        }
    },
    value: {
        integration_id: Id<IntegrationMarker>
    }
);
cmd::impl_str_wrapper_methods_with_two_id!(
    guild_integration,
    guild_id,
    integration_id,
    GuildMarker,
    IntegrationMarker,
    GuildIntegration
);

impl<S: CacheStrategy> Pipe<S> {
    /// Overwrite guild integration with new data.
    pub(crate) fn update_guild_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
        integration: &S::GuildIntegration,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::GuildIntegration {
                guild_id,
                id: integration_id,
            },
            integration.to_redis_arg()?,
        );
        Ok(self)
    }

    pub(crate) fn add_guild_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
        integration: &S::GuildIntegration,
    ) -> Result<&mut Self, Error> {
        self.0
            .sadd(
                RedisKey::GuildIntegrationId { guild_id },
                integration_id.get(),
            )
            .set(
                RedisKey::GuildIntegration {
                    guild_id,
                    id: integration_id,
                },
                integration.to_redis_arg()?,
            );

        Ok(self)
    }

    pub(crate) fn remove_guild_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
    ) -> &mut Self {
        self.0
            .srem(
                RedisKey::GuildIntegrationId { guild_id },
                integration_id.get(),
            )
            .del(RedisKey::GuildIntegration {
                guild_id,
                id: integration_id,
            });

        self
    }
}
