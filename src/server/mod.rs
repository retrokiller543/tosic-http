use crate::error::{Error, ServerError};
use crate::traits::handler::Handler;
use std::sync::Arc;
use tokio::io;
use tokio::net::unix::SocketAddr;
use tokio::net::ToSocketAddrs;
use tracing::{error, info, trace};

pub mod builder;

pub struct HttpServer {
    listener: tokio::net::TcpListener,
}

impl HttpServer {
    pub async fn new<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;

        trace!("Server Bound to {}", listener.local_addr()?);

        Ok(Self { listener })
    }

    pub async fn serve(self) -> Result<(), ServerError> {
        info!("Listening on {}", self.listener.local_addr()?);
        loop {
            match self.listener.accept().await {
                Ok((stream, socket)) => {
                    trace!("Accepted connection from {}", socket);
                    self.accept_connection(stream, socket)?;
                }
                Err(err) => {
                    error!("Failed to accept connection: {}", err);
                    continue;
                }
            }
        }
    }

    fn accept_connection(
        &self,
        stream: tokio::net::TcpStream,
        socket: std::net::SocketAddr,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
