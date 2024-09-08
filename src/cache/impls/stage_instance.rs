use twilight_model::id::{
    marker::{GuildMarker, StageMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, WithGuildId},
    CacheStrategy, Connection, Error,
};

cmd::impl_set_wrapper_methods!(
    guild_stage_instance_id,
    key: {
        RedisKey::GuildStageInstanceId: {
            guild_id: Id<GuildMarker>
        }
    },
    value: { stage_id: Id<StageMarker> }
);
cmd::impl_str_wrapper_methods!(
    stage_instance,
    key: { stage_id: Id<StageMarker> },
    value: WithGuildId<S::StageInstance>
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_guild_stage_instance_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        stage_id: Id<StageMarker>,
    ) -> &mut Self {
        self.0
            .sadd(RedisKey::GuildStageInstanceId { guild_id }, stage_id.get());
        self
    }

    pub(crate) fn remove_guild_stage_instance_id(
        &mut self,
        guild_id: Id<GuildMarker>,
        stage_id: Id<StageMarker>,
    ) -> &mut Self {
        self.0
            .srem(RedisKey::GuildStageInstanceId { guild_id }, stage_id.get());
        self
    }

    pub(crate) fn set_stage_instance(
        &mut self,
        guild_id: Id<GuildMarker>,
        stage_id: Id<StageMarker>,
        stage_instance: &S::StageInstance,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::from(stage_id),
            WithGuildId::to_bytes(guild_id, stage_instance)?,
        );
        Ok(self)
    }

    pub(crate) fn delete_stage_instance(&mut self, stage_id: Id<StageMarker>) -> &mut Self {
        self.0.del(RedisKey::from(stage_id));
        self
    }
}
