use twilight_model::{
    channel::StageInstance,
    gateway::payload::incoming::{StageInstanceCreate, StageInstanceDelete, StageInstanceUpdate},
    id::{
        marker::{GuildMarker, StageMarker},
        Id,
    },
};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

pub fn cache_stage_instance<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    stage_instance: StageInstance,
) -> Result<(), Error> {
    pipe.add_guild_stage_instance_id(stage_instance.guild_id, stage_instance.id)
        .set_stage_instance(
            stage_instance.guild_id,
            stage_instance.id,
            &S::StageInstance::from(stage_instance),
        )?;
    Ok(())
}

pub fn uncache_stage_instance<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    stage_id: Id<StageMarker>,
) {
    pipe.remove_guild_stage_instance_id(guild_id, stage_id)
        .delete_stage_instance(stage_id);
}

impl<S: CacheStrategy> UpdateCache<S> for StageInstanceCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::STAGE_INSTANCE) {
            cache_stage_instance(pipe, self.0.clone())?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for StageInstanceDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::STAGE_INSTANCE) {
            uncache_stage_instance(pipe, self.guild_id, self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for StageInstanceUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::STAGE_INSTANCE) {
            cache_stage_instance(pipe, self.0.clone())?;
        }

        Ok(())
    }
}
