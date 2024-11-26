use std::sync::OnceLock;

use redis::Client;
use tokio::runtime::Runtime;

use crate::{Config, ConnectionDriver, DefaultCacheStrategy, RedisCache};

pub(crate) fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();

    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap()
    })
}

pub(crate) fn block_on(future: impl std::future::Future) {
    runtime().block_on(future);
}

pub(crate) async fn redis_cache() -> RedisCache<DefaultCacheStrategy> {
    static REDIS: OnceLock<Client> = OnceLock::new();

    let url = option_env!("TEST_REDIS_URL").unwrap_or("redis://127.0.0.1");
    let client = REDIS.get_or_init(|| Client::open(url).unwrap());

    RedisCache::new(
        ConnectionDriver::MultiplexedClone(
            client.get_multiplexed_tokio_connection().await.unwrap(),
        ),
        Config::default(),
    )
}

pub mod model {
    use twilight_model::{id::Id, user::CurrentUser};

    pub fn current_user() -> CurrentUser {
        CurrentUser {
            accent_color: Some(0x0),
            avatar: None,
            banner: None,
            bot: true,
            discriminator: 1234,
            email: None,
            id: Id::new(1),
            mfa_enabled: true,
            name: "current".to_owned(),
            verified: Some(false),
            premium_type: None,
            public_flags: None,
            flags: None,
            locale: None,
        }
    }
}
