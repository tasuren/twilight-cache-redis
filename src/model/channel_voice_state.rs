use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::traits::CacheableChannelVoiceState;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CachedChannelVoiceState {
    pub guild_id: Id<GuildMarker>,
    pub user_id: Id<UserMarker>,
}

impl From<(Id<GuildMarker>, Id<UserMarker>)> for CachedChannelVoiceState {
    fn from((guild_id, user_id): (Id<GuildMarker>, Id<UserMarker>)) -> Self {
        Self { guild_id, user_id }
    }
}

crate::cache::value::impl_from_bytes_for_model!(CachedChannelVoiceState);
crate::cache::value::impl_to_bytes_for_model!(CachedChannelVoiceState);

impl CacheableChannelVoiceState for CachedChannelVoiceState {}
