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
mod sticker;
mod thread;
mod user;
mod voice_state;

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

#[cfg(test)]
mod tests {
    use redis_test::{MockCmd, MockRedisConnection};
    use twilight_model::{
        gateway::payload::incoming::Ready,
        id::Id,
        oauth::{ApplicationFlags, PartialApplication},
    };

    use crate::test;

    #[test]
    fn ready() {
        test::block_on(async {
            let mut cache = test::redis_cache().await;

            let event = Ready {
                user: test::model::current_user(),
                application: PartialApplication {
                    flags: ApplicationFlags::empty(),
                    id: Id::new(0),
                },
                guilds: Vec::new(),
                resume_gateway_url: String::new(),
                session_id: String::new(),
                shard: None,
                version: 1,
            };

            cache.update(event).await.unwrap();
        });
    }
}
