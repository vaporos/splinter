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

use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

use mio::{net::TcpStream as MioTcpStream, Evented};
use websocket::client::sync::Client;
use websocket::message::{Message, OwnedMessage::Binary};
use websocket::result::WebSocketError;
use websocket::stream::sync::{AsTcpStream, Stream};

use crate::transport::{Connection, DisconnectError, RecvError, SendError};

pub(super) struct WsClientConnection<S>
where
    S: Read + Write + Send,
{
    client: Client<S>,
    mio_stream: MioTcpStream,
}

impl<S> WsClientConnection<S>
where
    S: Read + Write + Send,
{
    pub fn new(client: Client<S>, mio_stream: MioTcpStream) -> Self {
        WsClientConnection { client, mio_stream }
    }
}

impl<S> Connection for WsClientConnection<S>
where
    S: AsTcpStream + Stream + Read + Write + Send,
{
    fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        Ok(self.client.send_message(&Message::binary(message))?)
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        loop {
            match self.client.recv_message() {
                Ok(message) => match message {
                    Binary(v) => break Ok(v),
                    _ => unimplemented!(),
                },
                Err(WebSocketError::IoError(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(WebSocketError::NoDataAvailable) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(err) => break Err(err.into()),
            }
        }
    }

    fn remote_endpoint(&self) -> String {
        format!("ws://{}", self.client.peer_addr().unwrap())
    }

    fn local_endpoint(&self) -> String {
        format!("ws://{}", self.client.local_addr().unwrap())
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        self.client.shutdown().map_err(DisconnectError::from)
    }

    fn evented(&self) -> &dyn Evented {
        &self.mio_stream
    }
}

impl From<WebSocketError> for RecvError {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::IoError(e) => RecvError::from(e),
            _ => RecvError::ProtocolError(format!("WebSocketError: {}", err.to_string())),
        }
    }
}

impl From<WebSocketError> for SendError {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::IoError(e) => SendError::from(e),
            _ => SendError::ProtocolError(format!("WebSocketError: {}", err.to_string())),
        }
    }
}
