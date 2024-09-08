use std::collections::VecDeque;

use redis::AsyncCommands;
use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker},
    Id,
};

use crate::{
    cache::{cmd, FromCachedRedisValue, Pipe, RedisKey, ToBytes},
    CacheStrategy, Connection, Error, RedisCache,
};

impl<S: CacheStrategy> RedisCache<S> {
    pub async fn len_channel_message_ids(
        &self,
        conn: &mut Connection<'_>,
        channel_id: Id<ChannelMarker>,
    ) -> Result<usize, Error> {
        Ok(conn.llen(RedisKey::ChannelMessageId { channel_id }).await?)
    }

    pub async fn index_channel_message_ids(
        &self,
        conn: &mut Connection<'_>,
        channel_id: Id<ChannelMarker>,
        index: isize,
    ) -> Result<Option<S::Message>, Error> {
        let raw: redis::Value = conn
            .lindex(RedisKey::ChannelMessageId { channel_id }, index)
            .await?;
        Option::from_cached_redis_value(&raw)
    }

    pub async fn range_channel_message_ids(
        &self,
        conn: &mut Connection<'_>,
        channel_id: Id<ChannelMarker>,
        start: isize,
        stop: isize,
    ) -> Result<VecDeque<S::Message>, Error> {
        let raw: redis::Value = conn
            .lrange(RedisKey::ChannelMessageId { channel_id }, start, stop)
            .await?;

        VecDeque::from_cached_redis_value(&raw)
    }
}

impl<S: CacheStrategy> Pipe<S> {
    pub fn len_channel_message_ids(&mut self, channel_id: Id<ChannelMarker>) -> &mut Self {
        self.0.llen(RedisKey::ChannelMessageId { channel_id });
        self
    }

    pub fn index_channel_message_ids(
        &mut self,
        channel_id: Id<ChannelMarker>,
        index: isize,
    ) -> &mut Self {
        self.0
            .lindex(RedisKey::ChannelMessageId { channel_id }, index);
        self
    }
}

cmd::impl_str_wrapper_methods!(
    message,
    key: { message_id: Id<MessageMarker> },
    value: S::Message
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn push_channel_message_id(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> &mut Self {
        self.0
            .rpush(RedisKey::ChannelMessageId { channel_id }, message_id.get());
        self
    }

    pub(crate) fn pop_channel_message_id(&mut self, channel_id: Id<ChannelMarker>) -> &mut Self {
        self.0.lpop(RedisKey::ChannelMessageId { channel_id }, None);
        self
    }

    pub(crate) fn remove_channel_message_id(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> &mut Self {
        self.0.lrem(
            RedisKey::ChannelMessageId { channel_id },
            0,
            message_id.get(),
        );
        self
    }

    pub(crate) fn set_message(
        &mut self,
        message_id: Id<MessageMarker>,
        message: &S::Message,
    ) -> Result<&mut Self, Error> {
        self.0.set(RedisKey::from(message_id), message.to_bytes()?);

        Ok(self)
    }

    pub(crate) fn delete_message(&mut self, message_id: Id<MessageMarker>) -> &mut Self {
        self.0.del(RedisKey::from(message_id));
        self
    }
}
