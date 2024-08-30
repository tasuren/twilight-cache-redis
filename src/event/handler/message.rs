use twilight_model::{
    gateway::payload::incoming::{MessageCreate, MessageDelete, MessageDeleteBulk},
    id::{
        marker::{ChannelMarker, MessageMarker},
        Id,
    },
};

use crate::{cache::Pipe, config::ResourceType, CacheStrategy, Error, RedisCache, UpdateCache};

fn uncache_message<S: CacheStrategy>(
    pipe: &mut Pipe<S>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) {
    pipe.delete_message(message_id)
        .remove_channel_message_id(channel_id, message_id);
}

impl<S: CacheStrategy> UpdateCache<S> for MessageCreate {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::USER) {
            crate::event::user::cache_user(pipe, self.author.clone(), self.guild_id)?;
        }

        if let (true, Some(member), Some(guild_id)) = (
            cache.wants(ResourceType::MEMBER),
            &self.member,
            self.guild_id,
        ) {
            super::member::cache_partial_member(pipe, guild_id, self.author.id, member.clone())?;
        }

        if !cache.wants(ResourceType::MESSAGE) {
            return Ok(());
        }

        let (cache_size, oldest_id): (usize, Option<Id<MessageMarker>>) = Pipe::<S>::new()
            .len_channel_message_ids(self.channel_id)
            .index_channel_message_ids(self.channel_id, 0)
            .query(&mut cache.get_connection().await?)
            .await?;

        if cache_size >= cache.config.message_cache_size {
            if let Some(oldest_id) = oldest_id {
                pipe.pop_channel_message_id(self.channel_id)
                    .delete_message(oldest_id);
            }
        }

        pipe.push_channel_message_id(self.channel_id, self.id)
            .set_message(self.id, &S::Message::from(self.0.clone()))?;

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for MessageDelete {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::MESSAGE) {
            uncache_message(pipe, self.channel_id, self.id);
        }

        Ok(())
    }
}

impl<S: CacheStrategy> UpdateCache<S> for MessageDeleteBulk {
    async fn update(&self, cache: &mut RedisCache<S>, pipe: &mut Pipe<S>) -> Result<(), Error> {
        if cache.wants(ResourceType::MESSAGE) {
            for id in self.ids.iter() {
                uncache_message(pipe, self.channel_id, *id);
            }
        }

        Ok(())
    }
}
