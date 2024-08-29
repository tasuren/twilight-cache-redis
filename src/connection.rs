use std::marker::PhantomData;

#[cfg(feature = "bb8")]
use bb8_redis::bb8::{Pool, PooledConnection};
use redis::aio::MultiplexedConnection;

use crate::Error;

pub enum Connection<'a> {
    MultiplexedConnection(MultiplexedConnection),
    #[cfg(feature = "bb8")]
    BB8PooledConnection(PooledConnection<'a, bb8_redis::RedisConnectionManager>),
    #[allow(missing_docs)]
    _RedisCacheReference(PhantomData<&'a ()>),
}

pub enum ConnectionDriver {
    MultiplexedClone(MultiplexedConnection),
    #[cfg(feature = "bb8")]
    BB8(Pool<bb8_redis::RedisConnectionManager>),
}

impl ConnectionDriver {
    pub async fn get(&self) -> Result<Connection<'_>, Error> {
        Ok(match self {
            ConnectionDriver::MultiplexedClone(conn) => {
                Connection::MultiplexedConnection(conn.clone())
            }
            #[cfg(feature = "bb8")]
            ConnectionDriver::BB8(pool) => Connection::BB8PooledConnection(pool.get().await?),
        })
    }
}

impl<'b> redis::aio::ConnectionLike for Connection<'b> {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        match self {
            Connection::MultiplexedConnection(conn) => conn.req_packed_command(cmd),
            #[cfg(feature = "bb8")]
            Connection::BB8PooledConnection(conn) => conn.req_packed_command(cmd),
            Connection::_RedisCacheReference(_) => unreachable!(),
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        let future = match self {
            Connection::MultiplexedConnection(conn) => conn.req_packed_commands(cmd, offset, count),
            #[cfg(feature = "bb8")]
            Connection::BB8PooledConnection(conn) => conn.req_packed_commands(cmd, offset, count),
            Connection::_RedisCacheReference(_) => unreachable!(),
        };
        Box::pin(future)
    }

    fn get_db(&self) -> i64 {
        match self {
            Connection::MultiplexedConnection(conn) => conn.get_db(),
            #[cfg(feature = "bb8")]
            Connection::BB8PooledConnection(conn) => conn.get_db(),
            Connection::_RedisCacheReference(_) => unreachable!(),
        }
    }
}
