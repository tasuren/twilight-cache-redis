use twilight_model::id::{
    marker::{GuildMarker, IntegrationMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    traits::CacheStrategy,
    Connection, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_integrations,
    key: {
        RedisKey::GuildIntegrations: {
            guild_id: Id<GuildMarker>
        }
    },
    value: {
        integration_id: Id<IntegrationMarker>
    }
);
cmd::impl_str_wrapper_methods_with_two_id!(
    guild_integration,
    key: {
        RedisKey::Integration: {
            guild_id: Id<GuildMarker>,
            integration_id: Id<IntegrationMarker>
        }
    },
    value: S::GuildIntegration
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
    ) -> &mut Self {
        self.0.sadd(
            RedisKey::GuildIntegrations { guild_id },
            integration_id.get(),
        );
        self
    }

    pub(crate) fn remove_guild_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
    ) -> &mut Self {
        self.0.srem(
            RedisKey::GuildIntegrations { guild_id },
            integration_id.get(),
        );

        self
    }

    pub(crate) fn set_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
        integration: &S::GuildIntegration,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::Integration {
                guild_id,
                integration_id,
            },
            integration.to_bytes()?,
        );

        Ok(self)
    }

    pub(crate) fn delete_integration(
        &mut self,
        guild_id: Id<GuildMarker>,
        integration_id: Id<IntegrationMarker>,
    ) -> &mut Self {
        self.0.del(RedisKey::Integration {
            guild_id,
            integration_id,
        });

        self
    }
}
