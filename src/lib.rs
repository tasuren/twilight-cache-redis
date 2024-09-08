#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod cache;
mod config;
mod connection;
pub mod event;
mod model;
mod traits;

use std::marker::PhantomData;

use self::config::ResourceType;
pub use self::{
    config::Config,
    connection::{Connection, ConnectionDriver},
    traits::CacheStrategy,
};

#[cfg(feature = "bb8")]
pub use bb8_redis;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User defined error is raised: {0}")]
    User(#[from] anyhow::Error),
    #[error("Failed to process data during bincode.")]
    Bincode(#[from] bincode::Error),
    #[error("Redis has raise error: {0}")]
    Redis(#[from] redis::RedisError),
    #[cfg(feature = "bb8")]
    #[error("Failed to acquire connection from bb8 pool.")]
    BB8(#[from] bb8_redis::bb8::RunError<redis::RedisError>),
    #[error("Failed to parse data. {msg} (Response was {response:?})")]
    Parse { msg: String, response: String },
}

mod private {
    use twilight_model::gateway::payload::incoming::*;

    pub trait Sealed {}

    impl Sealed for ChannelCreate {}
    impl Sealed for ChannelDelete {}
    impl Sealed for ChannelPinsUpdate {}
    impl Sealed for ChannelUpdate {}
    impl Sealed for GuildCreate {}
    impl Sealed for GuildEmojisUpdate {}
    impl Sealed for GuildDelete {}
    impl Sealed for GuildStickersUpdate {}
    impl Sealed for GuildUpdate {}
    impl Sealed for IntegrationCreate {}
    impl Sealed for IntegrationDelete {}
    impl Sealed for IntegrationUpdate {}
    impl Sealed for InteractionCreate {}
    impl Sealed for MemberAdd {}
    impl Sealed for MemberChunk {}
    impl Sealed for MemberRemove {}
    impl Sealed for MemberUpdate {}
    impl Sealed for MessageCreate {}
    impl Sealed for MessageDelete {}
    impl Sealed for MessageDeleteBulk {}
    impl Sealed for MessageUpdate {}
    impl Sealed for PresenceUpdate {}
    impl Sealed for ReactionAdd {}
    impl Sealed for ReactionRemove {}
    impl Sealed for ReactionRemoveAll {}
    impl Sealed for ReactionRemoveEmoji {}
    impl Sealed for Ready {}
    impl Sealed for RoleCreate {}
    impl Sealed for RoleDelete {}
    impl Sealed for RoleUpdate {}
    impl Sealed for StageInstanceCreate {}
    impl Sealed for StageInstanceDelete {}
    impl Sealed for StageInstanceUpdate {}
    impl Sealed for ThreadCreate {}
    impl Sealed for ThreadDelete {}
    impl Sealed for ThreadListSync {}
    impl Sealed for ThreadUpdate {}
    impl Sealed for UnavailableGuild {}
    impl Sealed for UserUpdate {}
    impl Sealed for VoiceStateUpdate {}
}

#[trait_variant::make(Send)]
pub trait UpdateCache<S: CacheStrategy>: private::Sealed {
    async fn update(
        &self,
        cache: &mut RedisCache<S>,
        pipe: &mut cache::Pipe<S>,
    ) -> Result<(), Error>;
}

pub struct DefaultCacheConfig;

impl CacheStrategy for DefaultCacheConfig {
    type SerdeError = bincode::Error;

    type Channel = twilight_model::channel::Channel;
    type ChannelVoiceState = model::CachedChannelVoiceState;
    type CurrentUser = twilight_model::user::CurrentUser;
    type Emoji = model::CachedEmoji;
    type Guild = model::CachedGuild;
    type GuildIntegration = twilight_model::guild::GuildIntegration;
    type Member = model::CachedMember;
    type Message = model::CachedMessage;
    type Presence = model::CachedPresence;
    type Role = twilight_model::guild::Role;
    type StageInstance = twilight_model::channel::StageInstance;
    type Sticker = model::CachedSticker;
    type User = twilight_model::user::User;
    type VoiceState = model::CachedVoiceState;
}

pub struct RedisCache<S: CacheStrategy> {
    connection_driver: ConnectionDriver,
    config: Config,
    _strategy: PhantomData<S>,
}

impl<S: CacheStrategy> RedisCache<S> {
    pub fn new(connection_driver: ConnectionDriver) -> Self {
        Self {
            connection_driver,
            config: Config::default(),
            _strategy: PhantomData,
        }
    }

    pub(crate) async fn get_connection(&self) -> Result<Connection<'_>, Error> {
        self.connection_driver.get().await
    }

    pub fn wants(&self, resource_type: ResourceType) -> bool {
        self.config.resource_types.contains(resource_type)
    }

    pub fn wants_any(&self, resource_type: ResourceType) -> bool {
        self.config.resource_types.intersects(resource_type)
    }

    pub async fn update(&mut self, cache: impl UpdateCache<S>) -> Result<(), Error> {
        let mut pipe = cache::Pipe::new();
        if self.config.atomic {
            pipe.atomic();
        }

        cache.update(self, &mut pipe).await?;

        if pipe.is_empty() {
            pipe.query(&mut self.get_connection().await?).await?;
        }

        Ok(())
    }
}
