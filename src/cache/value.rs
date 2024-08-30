use std::collections::VecDeque;

use atoi::atoi;
use redis::{FromRedisValue, Value};
use serde::de::DeserializeOwned;

use twilight_model::id::Id;

use crate::Error;

pub trait FromCachedRedisValue: Sized {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error>;
}

pub(crate) fn deserialize_single<T: DeserializeOwned>(value: &Value) -> Result<T, Error> {
    match value {
        Value::BulkString(data) => Ok(bincode::deserialize(data)?),
        _ => Err(Error::Parse {
            msg: "The value is not bytes.".to_owned(),
            response: format!("{value:?}"),
        }),
    }
}

macro_rules! impl_from_cached_redis_value_for_number {
    ($($num:ty),* $(,)?) => {
        $(
            impl FromCachedRedisValue for $num {
                fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
                    Ok(Self::from_redis_value(value)?)
                }
            }
        )*
    };
}

impl_from_cached_redis_value_for_number!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl FromCachedRedisValue for bool {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        Ok(FromRedisValue::from_redis_value(value)?)
    }
}

impl<T: FromCachedRedisValue> FromCachedRedisValue for Option<T> {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        match value {
            Value::Nil => Ok(None),
            _ => T::from_cached_redis_value(value).map(Some),
        }
    }
}

#[macro_export]
macro_rules! impl_from_cached_redis_value_for_model {
    ($($model:ty),*) => {
        $(
            impl $crate::cache::FromCachedRedisValue for $model {
                fn from_cached_redis_value(value: &redis::Value) -> Result<Self, $crate::Error> {
                    $crate::cache::value::deserialize_single(&value)
                }
            }
        )*
    };
}

pub use impl_from_cached_redis_value_for_model;

fn id_from_bytes<M>(raw: &[u8]) -> Result<Id<M>, Error> {
    let n = atoi(raw).ok_or_else(|| Error::Parse {
        msg: "Failed to parse ID.".to_owned(),
        response: format!("{raw:?}"),
    })?;
    Ok(Id::new(n))
}

impl<M> FromCachedRedisValue for Id<M> {
    fn from_cached_redis_value(value: &Value) -> Result<Id<M>, Error> {
        match value {
            Value::BulkString(raw) => id_from_bytes(raw),
            _ => Err(Error::Parse {
                msg: "The value is not an integer.".to_owned(),
                response: format!("{value:?}"),
            }),
        }
    }
}

impl<T: FromCachedRedisValue> FromCachedRedisValue for Vec<T> {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        match value {
            Value::Array(values) => values
                .iter()
                .map(|v| T::from_cached_redis_value(v))
                .collect(),
            _ => Err(Error::Parse {
                msg: "The value is not Array.".to_owned(),
                response: format!("{value:?}"),
            }),
        }
    }
}

impl<T: FromCachedRedisValue> FromCachedRedisValue for VecDeque<T> {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        Vec::from_cached_redis_value(value).map(VecDeque::from)
    }
}

macro_rules! impl_drv_for_tuple {
    ($n:expr) => { seq_macro::seq!(N in 0..$n {
        impl<#(T~N,)*> FromCachedRedisValue for (#(T~N,)*)
        where #(T~N: FromCachedRedisValue,)*
        {
            fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
                let Value::Array(values) = value else {
                    return Err(Error::Parse {
                        msg: "The value is not Array.".to_owned(),
                        response: format!("{value:?}"),
                    })
                };

                let data = (#(
                    T~N::from_cached_redis_value(
                        values
                            .get(N)
                            .ok_or_else(|| Error::Parse {
                                msg: "Insufficient elements in array.".to_owned(),
                                response: format!("{values:?}"),
                            })?
                    )?,
                )*);

                if values.is_empty() {
                    Ok(data)
                } else {
                    Err(Error::Parse {
                        msg: "Excessive elements in array.".to_owned(),
                        response: format!("{values:?}"),
                    })
                }
            }
        }
    }); };
}

seq_macro::seq!(I in 0..16 {
    impl_drv_for_tuple!(I);
});

pub trait ToCachedRedisArg {
    fn to_redis_arg(&self) -> Result<Vec<u8>, Error>;
}

#[macro_export]
macro_rules! impl_to_cached_redis_arg_for_model {
    ($($model:ty),*) => {
        $(
            impl $crate::cache::ToCachedRedisArg for $model {
                fn to_redis_arg(&self) -> Result<Vec<u8>, $crate::Error> {
                    Ok(::bincode::serialize(self)?)
                }
            }
        )*
    };
}
