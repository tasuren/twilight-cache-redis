use twilight_model::gateway::payload::incoming::{Ready, UserUpdate};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

mod channel;
mod emoji;
mod guild;
mod integration;
mod interaction;
mod member;
mod message;
mod presence;
mod reaction;
mod role;
mod stage_instance;

impl<S: CacheStrategy> UpdateCache<S> for Ready {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::USER_CURRENT) {
            pipe.set_current_user(&S::CurrentUser::from(self.user.clone()))?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for UserUpdate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::USER_CURRENT) {
            pipe.set_current_user(&S::CurrentUser::from(self.0.clone()))?;
        }

        Ok(())
    }
}
