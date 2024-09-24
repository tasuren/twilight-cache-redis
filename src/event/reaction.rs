use twilight_model::{
    channel::message::{Reaction, ReactionType},
    gateway::payload::incoming::{
        ReactionAdd, ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji,
    },
};

use crate::{
    cache::Pipe,
    config::ResourceType,
    traits::{CacheableCurrentUser, CacheableMessage},
    CacheStrategy, Error, RedisCache, UpdateCache,
};

fn reactions_eq(a: &ReactionType, b: &ReactionType) -> bool {
    match (a, b) {
        (ReactionType::Custom { id: id_a, .. }, ReactionType::Custom { id: id_b, .. }) => {
            id_a == id_b
        }
        (ReactionType::Unicode { name: name_a }, ReactionType::Unicode { name: name_b }) => {
            name_a == name_b
        }
        _ => false,
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ReactionAdd {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::REACTION) {
            return Ok(());
        };

        let mut conn = cache.get_connection().await?;
        let Some(mut message) = cache.get_message(&mut conn, self.message_id).await? else {
            return Ok(());
        };

        if let Some(reaction) = message
            .reactions_mut()
            .iter_mut()
            .find(|reaction| reactions_eq(&reaction.emoji, &self.emoji))
        {
            if !reaction.me {
                if let Some(current_user) = cache.get_current_user(&mut conn).await? {
                    if current_user.id() == self.user_id {
                        reaction.me = true;
                    }
                }
            }

            reaction.count += 1;
        } else {
            let me = cache
                .get_current_user(&mut conn)
                .await?
                .is_some_and(|current_user| current_user.id() == self.user_id);

            message.add_reaction(Reaction {
                count: 1,
                emoji: self.emoji.clone(),
                me,
            });
        }

        pipe.set_message(self.message_id, &message)?;

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ReactionRemove {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::REACTION) {
            return Ok(());
        };

        let mut conn = cache.get_connection().await?;
        let Some(mut message) = cache.get_message(&mut conn, self.message_id).await? else {
            return Ok(());
        };

        if let Some(reaction) = message
            .reactions_mut()
            .iter_mut()
            .find(|reaction| reactions_eq(&reaction.emoji, &self.emoji))
        {
            if reaction.me {
                if let Some(current_user) = cache.get_current_user(&mut conn).await? {
                    if current_user.id() == self.user_id {
                        reaction.me = false;
                    }
                }
            }

            if reaction.count > 1 {
                reaction.count -= 1;
            } else {
                message.retain_reactions(|reaction| !reactions_eq(&reaction.emoji, &self.emoji));
            }

            pipe.set_message(self.message_id, &message)?;
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ReactionRemoveAll {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::REACTION) {
            return Ok(());
        };

        let mut conn = cache.get_connection().await?;
        let Some(mut message) = cache.get_message(&mut conn, self.message_id).await? else {
            return Ok(());
        };

        message.clear_reactions();
        pipe.set_message(self.message_id, &message)?;

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for ReactionRemoveEmoji {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if !cache.wants(ResourceType::REACTION) {
            return Ok(());
        };

        let mut conn = cache.get_connection().await?;
        let Some(mut message) = cache.get_message(&mut conn, self.message_id).await? else {
            return Ok(());
        };

        if let Some(index) = message
            .reactions()
            .iter()
            .position(|reaction| reactions_eq(&reaction.emoji, &self.emoji))
        {
            message.remove_reaction(index);
            pipe.set_message(self.message_id, &message)?;
        }

        Ok(())
    }
}
