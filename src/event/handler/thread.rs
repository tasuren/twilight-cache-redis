use twilight_model::gateway::payload::incoming::{
    ThreadCreate, ThreadDelete, ThreadListSync, ThreadUpdate,
};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

use super::channel::{cache_channel, uncache_channel};

impl<S: CacheStrategy> UpdateCache<S> for ThreadCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            cache_channel(pipe, self.0.clone())?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ThreadDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            uncache_channel(pipe, Some(self.guild_id), self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ThreadListSync {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            let threads = self.threads.clone();
            for thread in threads {
                cache_channel(pipe, thread)?;
            }
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ThreadUpdate {
    async fn update(
        &self,
        cache: &mut RedisCache<S>,
        pipe: &mut crate::cache::Pipe<S>,
    ) -> Result<(), Error> {
        if cache.wants(ResourceType::CHANNEL) {
            cache_channel(pipe, self.0.clone())?;
        }

        Ok(())
    }
}
