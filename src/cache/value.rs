use std::collections::VecDeque;

use atoi::atoi;
use redis::{FromRedisValue, ToRedisArgs, Value};

use twilight_model::id::Id;

use crate::Error;

pub trait FromBytes: Sized {
    fn from_bytes(raw: &[u8]) -> Result<Self, Error>;
}

/// Implement `FromBytes` for the given models that implement `serde::Deserialize`.
///
/// # Notes
/// This macro uses bincode crate to deserialize the data.
#[macro_export]
macro_rules! __impl_from_bytes_for_model {
    ($($model:ty),*) => {
        $(
            impl $crate::cache::value::FromBytes for $model {
                fn from_bytes(data: &[u8]) -> Result<Self, $crate::Error> {
                    Ok(::serde_json::from_slice(data)?)
                }
            }
        )*
    };
}

pub use __impl_from_bytes_for_model as impl_from_bytes_for_model;

impl<M> FromBytes for Id<M> {
    fn from_bytes(raw: &[u8]) -> Result<Self, Error> {
        let n = atoi(raw).ok_or_else(|| Error::Parse {
            msg: "Failed to parse ID.".to_owned(),
            response: format!("{raw:?}"),
        })?;
        Ok(Id::new(n))
    }
}

pub trait FromCachedRedisValue: Sized {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error>;
}

impl FromCachedRedisValue for Value {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        Ok(value.clone())
    }
}

impl<T: FromBytes> FromCachedRedisValue for T {
    fn from_cached_redis_value(value: &Value) -> Result<Self, Error> {
        match value {
            Value::BulkString(data) => Ok(T::from_bytes(data)?),
            _ => Err(Error::Parse {
                msg: "The value is not bytes.".to_owned(),
                response: format!("{value:?}"),
            }),
        }
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

                if values.len() <= $n {
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

pub trait ToBytes {
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
}

/// Implement `ToBytes` for the given models that implement `serde::Serialize`.
///
/// # Notes
/// This macro uses bincode crate to serialize the data.
#[macro_export]
macro_rules! __impl_to_bytes_for_model {
    ($($model:ty),*) => {
        $(
            impl $crate::cache::ToBytes for $model {
                fn to_bytes(&self) -> Result<Vec<u8>, $crate::Error> {
                    Ok(::serde_json::to_vec(self)?)
                }
            }
        )*
    };
}

pub use __impl_to_bytes_for_model as impl_to_bytes_for_model;

impl<M> ToBytes for Id<M> {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(self.get().to_redis_args().remove(0))
    }
}
