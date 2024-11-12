use crate::error::ServerError;
use tokio::io;
use tokio::net::ToSocketAddrs;
use tracing::{error, info, trace};
use crate::handlers::Handlers;

pub mod builder;
mod test;
pub struct HttpServer {
    listener: tokio::net::TcpListener,
    handlers: Handlers
}

impl HttpServer {
    pub async fn new<T: ToSocketAddrs>(addr: T, handlers: Handlers) -> io::Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;

        trace!("Server Bound to {}", listener.local_addr()?);

        Ok(Self { listener, handlers })
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

    #[allow(unused_variables)]
    fn accept_connection(
        &self,
        stream: tokio::net::TcpStream,
        socket: std::net::SocketAddr,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
