use redis::AsyncCommands;
use twilight_model::id::Id;

use super::{helper::IdAsyncIter, FromCachedRedisValue, Pipe};
use crate::{cache::RedisKey, CacheStrategy, Connection, Error};

/// Re-exported items for use in generated code by macros.
#[allow(unused_imports)]
pub(crate) mod __export {
    pub use paste::paste;
    pub use twilight_model::id::Id;

    pub use super::*;
    pub use crate::{
        cache::{helper::IdAsyncIter, RedisKey},
        CacheStrategy, Connection, Error,
    };
}

pub async fn scan<'a, 'stmt, VM>(
    conn: &'stmt mut Connection<'a>,
    key: RedisKey,
) -> Result<IdAsyncIter<'stmt, VM>, Error> {
    Ok(IdAsyncIter::new(conn.sscan(key).await?))
}

pub async fn contains<'a, VM>(
    conn: &mut Connection<'a>,
    key: RedisKey,
    value: Id<VM>,
) -> Result<bool, Error> {
    Ok(conn.sismember(key, value.get()).await?)
}

pub async fn len<'a>(conn: &mut Connection<'a>, key: RedisKey) -> Result<usize, Error> {
    Ok(conn.scard(key).await?)
}

pub fn contains_with_pipe<S: CacheStrategy, VM>(pipe: &mut Pipe<S>, key: RedisKey, value: Id<VM>) {
    pipe.0.sismember(key, value.get());
}

pub fn len_with_pipe<S: CacheStrategy>(pipe: &mut Pipe<S>, key: RedisKey) {
    pipe.0.scard(key);
}

/// Implement a getter for a set of values.
macro_rules! impl_set_wrapper_methods {
    (
        $set_name:ident,
        $redis_key:ident,
        $key_name:ident,
        $value_name:ident,
        $key_id_marker:ty,
        $value_id_marker:ty
    ) => {
        $crate::cache::cmd::__export::paste! {
        mod [<$set_name _set_wrapper_impl>] {
            use super::*;
            use $crate::cache::cmd::__export::*;

            impl<S: CacheStrategy> $crate::RedisCache<S> {
                pub async fn [<scan_ $set_name>]<'a, 'stmt>(
                    &'a self,
                    conn: &'stmt mut Connection<'a>,
                    $key_name: Id<$key_id_marker>,
                ) -> Result<
                    IdAsyncIter<'stmt, $value_id_marker>,
                    Error
                > {
                    scan(
                        conn,
                        RedisKey::$redis_key { $key_name }
                    ).await
                }

                pub async fn [<$set_name _contains>](
                    &self,
                    conn: &mut Connection<'_>,
                    $key_name: Id<$key_id_marker>,
                    $value_name: Id<$value_id_marker>,
                ) -> Result<bool, Error> {
                    contains(
                        conn,
                        RedisKey::$redis_key { $key_name },
                        $value_name
                    ).await
                }

                pub async fn [<len_ $set_name>](
                    &self,
                    conn: &mut Connection<'_>,
                    $key_name: Id<$key_id_marker>,
                ) -> Result<usize, Error> {
                    len(
                        conn,
                        RedisKey::$redis_key { $key_name }
                    ).await
                }
            }

            impl<S: CacheStrategy> $crate::cache::Pipe<S> {
                pub fn [<$set_name _contains>](
                    &mut self,
                    $key_name: Id<$key_id_marker>,
                    $value_name: Id<$value_id_marker>,
                ) -> &mut Self {
                    contains_with_pipe(
                        self,
                        RedisKey::$redis_key { $key_name },
                        $value_name
                    );
                    self
                }

                pub fn [<len_ $set_name>](
                    &mut self,
                    $key_name: Id<$key_id_marker>,
                ) -> &mut Self {
                    len_with_pipe(
                        self,
                        RedisKey::$redis_key { $key_name }
                    );
                    self
                }
            }
        } }
    };
}

/// Implement a getter for a set of values.
macro_rules! impl_global_set_wrapper_methods {
    (
        $set_name:ident,
        $redis_key:ident,
        $value_name:ident,
        $value_id_marker:ty
    ) => {
        $crate::cache::cmd::__export::paste! {
        mod [<$set_name _set_wrapper_impl>] {
            use super::*;
            use $crate::cache::cmd::__export::*;

            impl<S: CacheStrategy> $crate::RedisCache<S> {
                pub async fn [<scan_ $set_name>]<'a, 'stmt>(
                    &'a mut self,
                    conn: &'stmt mut Connection<'a>,
                ) -> Result<IdAsyncIter<'stmt, $value_id_marker>, Error> {
                    scan(
                        conn,
                        RedisKey::$redis_key
                    ).await
                }

                pub async fn [<$set_name _contains>](
                    &mut self,
                    conn: &mut Connection<'_>,
                    $value_name: Id<$value_id_marker>,
                ) -> Result<bool, Error> {
                    contains(
                        conn,
                        RedisKey::$redis_key,
                        $value_name
                    ).await
                }

                pub async fn [<len_ $set_name>](
                    &mut self,
                    conn: &mut Connection<'_>,
                ) -> Result<usize, Error> {
                    len(
                        conn,
                        RedisKey::$redis_key
                    ).await
                }
            }

            impl<S: CacheStrategy> $crate::cache::Pipe<S> {
                pub fn [<$set_name _contains>](
                    &mut self,
                    $value_name: Id<$value_id_marker>,
                ) -> &mut Self {
                    contains_with_pipe(
                        self,
                        RedisKey::$redis_key,
                        $value_name
                    );
                    self
                }

                pub fn [<len_ $set_name>](
                    &mut self,
                ) -> &mut Self {
                    len_with_pipe(
                        self,
                        RedisKey::$redis_key
                    );
                    self
                }
            }
        } }
    };
}

pub async fn get<'a, T: FromCachedRedisValue>(
    conn: &mut Connection<'a>,
    key: impl Into<RedisKey>,
) -> Result<Option<T>, Error> {
    let data: redis::Value = conn.get(key.into()).await?;
    T::from_cached_redis_value(&data).map(Some)
}

pub fn get_with_pipe<S: CacheStrategy>(pipe: &mut Pipe<S>, key: impl Into<RedisKey>) {
    pipe.0.get(key.into());
}

macro_rules! impl_str_wrapper_methods {
    (
        $get_name:ident,
        $key_name:ident,
        $value_name:ident,
        $value_id_marker:ty
    ) => {
        $crate::cache::cmd::__export::paste! {
        mod [< $get_name _str_wrapper_impl >] {
            use super::*;
            use $crate::cache::cmd::__export::*;

            impl<S: CacheStrategy> $crate::RedisCache<S> {
                pub async fn [<get_ $get_name>](
                    &self,
                    conn: &mut Connection<'_>,
                    $key_name: Id<$value_id_marker>,
                ) -> Result<Option<S::$value_name>, Error> {
                    get(conn, $key_name).await
                }
            }

            impl<S: CacheStrategy> $crate::cache::Pipe<S> {
                pub fn [<get_ $get_name>](
                    &mut self,
                    $key_name: Id<$value_id_marker>,
                ) -> &mut Self {
                    get_with_pipe(self, $key_name);
                    self
                }
            }
        } }
    };
}

macro_rules! impl_str_wrapper_methods_with_two_id {
    (
        $get_name:ident,
        $id_name:ident,
        $id2_name:ident,
        $id_marker:ty,
        $id2_marker:ty,
        $value_name:ident
    ) => {
        $crate::cache::cmd::__export::paste! {
        mod [< $get_name _str_wrapper_impl >] {
            use super::*;
            use $crate::cache::cmd::__export::*;

            impl<S: CacheStrategy> $crate::RedisCache<S> {
                pub async fn [<get_ $get_name>](
                    &self,
                    conn: &mut Connection<'_>,
                    $id_name: Id<$id_marker>,
                    $id2_name: Id<$id2_marker>
                ) -> Result<Option<S::$value_name>, Error> {
                    get(conn, ($id_name, $id2_name)).await
                }
            }

            impl<S: CacheStrategy> $crate::cache::Pipe<S> {
                pub fn [<get_ $get_name>](
                    &mut self,
                    $id_name: Id<$id_marker>,
                    $id2_name: Id<$id2_marker>
                ) -> &mut Self {
                    get_with_pipe(self, ($id_name, $id2_name));
                    self
                }
            }
        } }
    };
}

pub(crate) use impl_global_set_wrapper_methods;
pub(crate) use impl_set_wrapper_methods;
pub(crate) use impl_str_wrapper_methods;
pub(crate) use impl_str_wrapper_methods_with_two_id;