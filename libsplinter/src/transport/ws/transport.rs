// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use mio::net::TcpStream as MioTcpStream;
use websocket::result::WebSocketError;
use websocket::server::sync::Server;
use websocket::stream::sync::AsTcpStream as _;
use websocket::{server::NoTlsAcceptor, ClientBuilder};

use crate::transport::{ConnectError, Connection, ListenError, Listener, Transport};

use super::connection::WsClientConnection;
use super::listener::WsListener;

const PROTOCOL_PREFIX: &str = "ws://";

/// A WebSocket-based `Transport`.
///
/// Supports endpoints of the format `ws://ip_or_host:port`.
///
/// # Examples
///
/// To connect to the a remote endpoint, send a message, and receive a reply message:
///
/// ```rust,no_run
/// use splinter::transport::Transport as _;
/// use splinter::transport::ws::WsTransport;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut transport = WsTransport::default();
///
///     // Connect to a remote endpoint starting wtih `ws://`.
///     let mut connection = transport.connect("ws://127.0.0.1:5555")?;
///
///     // Send some bytes
///     connection.send(b"hello world")?;
///
///     // Receive a response
///     let msg = connection.recv()?;
///
///     // Disconnect
///     connection.disconnect()?;
///
///     Ok(())
/// }
/// ```
///
/// To accept a connection, receive, and send a reply:
///
/// ```rust, no_run
/// use splinter::transport::Transport as _;
/// use splinter::transport::ws::WsTransport;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut transport = WsTransport::default();
///
///     // Create a listener, which will bind to the port
///     let mut listener = transport.listen("ws://127.0.0.1:5555")?;
///
///     // When the other side connects, accept will return a `Connection`
///     let mut connection = listener.accept()?;
///
///     // Receive a message
///     let msg = connection.recv()?;
///
///     // Send a response
///     connection.send(b"hello world")?;
///
///     // Disconnect
///     connection.disconnect()?;
///
///     Ok(())
/// }
#[derive(Default)]
pub struct WsTransport {}

impl Transport for WsTransport {
    fn accepts(&self, address: &str) -> bool {
        address.starts_with(PROTOCOL_PREFIX)
    }

    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        if !self.accepts(endpoint) {
            return Err(ConnectError::ProtocolError(format!(
                "Invalid protocol \"{}\"",
                endpoint
            )));
        }

        let client = ClientBuilder::new(endpoint)?.connect_insecure()?;

        let stream_ref = client.stream_ref();
        let stream_clone = stream_ref.as_tcp().try_clone().unwrap();
        let mio_stream = MioTcpStream::from_stream(stream_clone).unwrap();

        Ok(Box::new(WsClientConnection::new(client, mio_stream)))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        if !self.accepts(bind) {
            return Err(ListenError::ProtocolError(format!(
                "Invalid protocol \"{}\"",
                bind
            )));
        }

        let address = if bind.starts_with(PROTOCOL_PREFIX) {
            &bind[PROTOCOL_PREFIX.len()..]
        } else {
            bind
        };

        let server: Server<NoTlsAcceptor> = Server::bind(address)?;

        Ok(Box::new(WsListener::new(server)))
    }
}

impl From<WebSocketError> for ConnectError {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::IoError(e) => ConnectError::from(e),
            _ => ConnectError::ProtocolError(format!("WebSocketError: {}", err.to_string())),
        }
    }
}
