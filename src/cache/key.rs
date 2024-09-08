use twilight_model::id::{
    marker::{
        self, ChannelMarker, EmojiMarker, GuildMarker, IntegrationMarker, MessageMarker,
        RoleMarker, StageMarker, StickerMarker, UserMarker,
    },
    Id,
};

#[derive(Debug, Clone, Copy)]
pub enum RedisKey {
    CurrentUser,
    Channel {
        id: Id<ChannelMarker>,
    },
    GuildChannels {
        guild_id: Id<GuildMarker>,
    },
    Emoji {
        id: Id<EmojiMarker>,
    },
    GuildEmojis {
        guild_id: Id<GuildMarker>,
    },
    Integration {
        guild_id: Id<GuildMarker>,
        id: Id<IntegrationMarker>,
    },
    GuildIntegrations {
        guild_id: Id<GuildMarker>,
    },
    User {
        id: Id<UserMarker>,
    },
    Users,
    UserGuilds {
        user_id: Id<UserMarker>,
    },
    Member {
        guild_id: Id<GuildMarker>,
        id: Id<UserMarker>,
    },
    GuildMembers {
        guild_id: Id<GuildMarker>,
    },
    UnavailableGuilds,
    Guild {
        id: Id<GuildMarker>,
    },
    Guilds,
    ChannelMessages {
        channel_id: Id<ChannelMarker>,
    },
    Message {
        id: Id<MessageMarker>,
    },
    GuildPresences {
        guild_id: Id<GuildMarker>,
    },
    Presence {
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    },
    GuildRoles {
        guild_id: Id<GuildMarker>,
    },
    Role {
        id: Id<RoleMarker>,
    },
    GuildStageInstances {
        guild_id: Id<GuildMarker>,
    },
    StageInstance {
        id: Id<StageMarker>,
    },
    GuildStickers {
        guild_id: Id<GuildMarker>,
    },
    Sticker {
        id: Id<StickerMarker>,
    },
    ChannelVoiceState {
        channel_id: Id<ChannelMarker>,
    },
    VoiceState {
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    },
}

macro_rules! impl_from_id {
    ($(($key_name:ident, $marker:ident)),* $(,)?) => {
        $(
            impl From<Id<marker::$marker>> for RedisKey {
                fn from(id: Id<twilight_model::id::marker::$marker>) -> Self {
                    RedisKey::$key_name { id }
                }
            }
        )*
    };
}

impl_from_id!(
    (Channel, ChannelMarker),
    (Emoji, EmojiMarker),
    (Guild, GuildMarker),
    (User, UserMarker),
    (Message, MessageMarker),
    (Role, RoleMarker),
    (StageInstance, StageMarker),
    (Sticker, StickerMarker)
);

macro_rules! impl_from_two_id {
    ($(
        (
            $key_name:ident, {
                $id_name:ident: $marker:ident,
                $id2_name:ident: $marker2:ident
            }
        )
    ),* $(,)?) => {
        $(
            impl From<(Id<marker::$marker>, Id<marker::$marker2>)> for RedisKey {
                fn from(($id_name, $id2_name): (Id<marker::$marker>, Id<marker::$marker2>)) -> Self {
                    RedisKey::$key_name { $id_name: $id_name, $id2_name: $id2_name }
                }
            }
        )*
    };
}

impl_from_two_id!(
    (Integration, {
        guild_id: GuildMarker,
        id: IntegrationMarker
    }),
    (Member, {
        guild_id: GuildMarker,
        id: UserMarker
    }),
);

enum KeyKind {
    Simple(&'static str),
    WithId((&'static str, u64)),
    WithGuildId((&'static str, u64, u64)),
}

impl From<&'static str> for KeyKind {
    fn from(key: &'static str) -> Self {
        KeyKind::Simple(key)
    }
}

impl<T> From<(&'static str, Id<T>)> for KeyKind {
    fn from((name, id): (&'static str, Id<T>)) -> Self {
        KeyKind::WithId((name, id.get()))
    }
}

impl<T> From<(&'static str, Id<GuildMarker>, Id<T>)> for KeyKind {
    fn from((name, guild_id, id): (&'static str, Id<GuildMarker>, Id<T>)) -> Self {
        KeyKind::WithGuildId((name, guild_id.get(), id.get()))
    }
}

impl From<KeyKind> for Vec<u8> {
    fn from(key: KeyKind) -> Vec<u8> {
        match key {
            KeyKind::Simple(key) => key.as_bytes().to_vec(),
            KeyKind::WithId((base, id)) => {
                let base = base.as_bytes();
                let mut buf = itoa::Buffer::new();
                let id = buf.format(id).as_bytes();

                let mut bytes = Vec::with_capacity(base.len() + 1 + id.len());

                bytes.extend_from_slice(base);
                bytes.push(b':');
                bytes.extend_from_slice(id);

                bytes
            }
            KeyKind::WithGuildId((base, guild_id, id)) => {
                let base = base.as_bytes();
                let mut buf = itoa::Buffer::new();
                let guild_id = buf.format(guild_id).as_bytes();
                let mut guild_buf = itoa::Buffer::new();
                let id = guild_buf.format(id).as_bytes();

                let mut bytes = Vec::with_capacity(base.len() + 1 + guild_id.len() + 1 + id.len());

                bytes.extend_from_slice(base);
                bytes.push(b':');
                bytes.extend_from_slice(guild_id);
                bytes.push(b':');
                bytes.extend_from_slice(id);

                bytes
            }
        }
    }
}

impl redis::ToRedisArgs for RedisKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: KeyKind = match self {
            Self::CurrentUser => "CURRENT_USER".into(),
            Self::Channel { id } => ("CHANNEL", *id).into(),
            Self::GuildChannels { guild_id } => ("GUILD_CHANNELS", *guild_id).into(),
            Self::Emoji { id } => ("EMOJI", *id).into(),
            Self::GuildEmojis { guild_id } => ("GUILD_EMOJIS", *guild_id).into(),
            Self::Integration { guild_id, id } => ("INTEGRATION", *guild_id, *id).into(),
            Self::GuildIntegrations { guild_id } => ("GUILD_INTEGRATIONS", *guild_id).into(),
            Self::User { id } => ("USER", *id).into(),
            Self::Users => "USERS".into(),
            Self::UserGuilds { user_id } => ("USER_GUILDS", *user_id).into(),
            Self::Member { guild_id, id } => ("MEMBER", *guild_id, *id).into(),
            Self::GuildMembers { guild_id } => ("GUILD_MEMBERS", *guild_id).into(),
            Self::UnavailableGuilds => "UNAVAILABLE_GUILDS".into(),
            Self::Guild { id } => ("GUILD", *id).into(),
            Self::Guilds => "GUILDS".into(),
            Self::ChannelMessages { channel_id } => ("CHANNEL_MESSAGES", *channel_id).into(),
            Self::Message { id } => ("MESSAGE", *id).into(),
            Self::GuildPresences { guild_id } => ("GUILD_PRESENCES", *guild_id).into(),
            Self::Presence { guild_id, user_id } => ("PRESENCE", *guild_id, *user_id).into(),
            Self::GuildRoles { guild_id } => ("GUILD_ROLES", *guild_id).into(),
            Self::Role { id } => ("ROLE", *id).into(),
            Self::GuildStageInstances { guild_id } => ("GUILD_STAGE_INSTANCES", *guild_id).into(),
            Self::StageInstance { id } => ("STAGE_INSTANCE", *id).into(),
            Self::GuildStickers { guild_id } => ("GUILD_STICKERS", *guild_id).into(),
            Self::Sticker { id } => ("STICKER", *id).into(),
            Self::ChannelVoiceState { channel_id } => ("VOICE_STATE_USER", *channel_id).into(),
            Self::VoiceState { guild_id, user_id } => ("VOICE_STATE", *guild_id, *user_id).into(),
        };

        let bytes: Vec<u8> = key.into();
        out.write_arg(&bytes);
    }
}
