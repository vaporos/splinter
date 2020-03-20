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

use std::net::TcpStream;

use mio::net::TcpStream as MioTcpStream;
use websocket::server::sync::Server;
use websocket::server::upgrade::sync::Buffer;
use websocket::server::{InvalidConnection, NoTlsAcceptor};
use websocket::stream::sync::AsTcpStream as _;

use crate::transport::{AcceptError, Connection, Listener};

use super::connection::WsClientConnection;

pub struct WsListener {
    server: Server<NoTlsAcceptor>,
}

impl WsListener {
    pub fn new(server: Server<NoTlsAcceptor>) -> Self {
        WsListener { server }
    }
}

impl Listener for WsListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        let client = self.server.accept()?.accept()?;

        let stream_ref = client.stream_ref();
        let stream_clone = stream_ref.as_tcp().try_clone().unwrap();
        let mio_stream = MioTcpStream::from_stream(stream_clone).unwrap();

        Ok(Box::new(WsClientConnection::new(client, mio_stream)))
    }

    fn endpoint(&self) -> String {
        format!("ws://{}", self.server.local_addr().unwrap())
    }
}

impl From<InvalidConnection<TcpStream, Buffer>> for AcceptError {
    fn from(iconn: InvalidConnection<TcpStream, Buffer>) -> Self {
        AcceptError::ProtocolError(format!("HyperIntoWsError: {}", iconn.error.to_string()))
    }
}

impl From<(TcpStream, std::io::Error)> for AcceptError {
    fn from(tuple: (TcpStream, std::io::Error)) -> Self {
        AcceptError::from(tuple.1)
    }
}
