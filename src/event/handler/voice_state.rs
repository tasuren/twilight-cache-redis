use twilight_model::{
    gateway::payload::incoming::VoiceStateUpdate,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
    voice::VoiceState,
};

use crate::{
    cache::Pipe, config::ResourceType, traits::CacheableVoiceState, CacheStrategy, Connection,
    Error, UpdateCache,
};

pub(crate) async fn cache_voice_state<S: CacheStrategy>(
    conn: &mut Connection<'_>,
    pipe: &mut Pipe<S>,
    guild_id: Id<GuildMarker>,
    voice_state: VoiceState,
) -> Result<(), Error> {
    let user_id = voice_state.user_id;

    // Check if the user is switching channels.
    // If they are, remove them from the old channel.
    let already_voice_state: Option<S::VoiceState> = Pipe::<S>::new()
        .get_voice_state(guild_id, user_id)
        .query(conn)
        .await?;

    if let Some(already_voice_state) = already_voice_state {
        pipe.remove_channel_voice_state(
            already_voice_state.channel_id(),
            &S::ChannelVoiceState::from((guild_id, user_id)),
        )?;
    }

    if let Some(channel_id) = voice_state.channel_id {
        // Cache the new voice state.
        let voice_state = S::VoiceState::from((channel_id, guild_id, voice_state));

        pipe.set_voice_state(guild_id, user_id, &voice_state)?
            .add_guild_voice_state(guild_id, user_id)
            .add_channel_voice_state(
                channel_id,
                &S::ChannelVoiceState::from((guild_id, user_id)),
            )?;
    } else {
        // This user is not in a voice channel so remove the cache.
        pipe.remove_guild_voice_state(guild_id, user_id)
            .delete_voice_state(guild_id, user_id);
    }

    Ok(())
}

impl<S: CacheStrategy> UpdateCache<S> for VoiceStateUpdate {
    async fn update(
        &self,
        cache: &mut crate::RedisCache<S>,
        pipe: &mut crate::cache::Pipe<S>,
    ) -> Result<(), Error> {
        if cache.wants(ResourceType::VOICE_STATE) {
            if let Some(guild_id) = self.guild_id {
                cache_voice_state(
                    &mut cache.get_connection().await?,
                    pipe,
                    guild_id,
                    self.0.clone(),
                )
                .await?;
            }
        }

        if cache.wants(ResourceType::MEMBER) {
            if let (Some(guild_id), Some(member)) = (self.guild_id, &self.member) {
                super::member::cache_member(pipe, guild_id, member.clone())?;
            }
        }

        Ok(())
    }
}
