use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

use crate::{
    cache::{cmd, Pipe, RedisKey, ToBytes},
    CacheStrategy, Error,
};

cmd::impl_set_wrapper_methods!(
    voice_state_user,
    key: {
        RedisKey::ChannelVoiceState: {
            channel_id: Id<ChannelMarker>
        }
    },
    value: { user: S::ChannelVoiceState }
);
cmd::impl_str_wrapper_methods!(
    voice_state,
    key: { channel_id: Id<ChannelMarker> },
    value: S::VoiceState
);

impl<S: CacheStrategy> Pipe<S> {
    pub(crate) fn add_channel_voice_state(
        &mut self,
        channel_id: Id<ChannelMarker>,
        user: &S::ChannelVoiceState,
    ) -> Result<&mut Self, Error> {
        self.0
            .sadd(RedisKey::ChannelVoiceState { channel_id }, user.to_bytes()?);

        Ok(self)
    }

    pub(crate) fn remove_voice_state_user(
        &mut self,
        channel_id: Id<ChannelMarker>,
        user: &S::ChannelVoiceState,
    ) -> Result<&mut Self, Error> {
        self.0
            .srem(RedisKey::ChannelVoiceState { channel_id }, user.to_bytes()?);

        Ok(self)
    }

    pub(crate) fn set_voice_state(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        voice_state: &S::VoiceState,
    ) -> Result<&mut Self, Error> {
        self.0.set(
            RedisKey::VoiceState { guild_id, user_id },
            voice_state.to_bytes()?,
        );

        Ok(self)
    }

    pub(crate) fn delete_voice_state(
        &mut self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> &mut Self {
        self.0.del(RedisKey::VoiceState { guild_id, user_id });

        self
    }
}
