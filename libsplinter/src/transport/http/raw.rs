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

use super::super::{
    AcceptError, ConnectError, Connection, DisconnectError, ListenError, Listener,
    RecvError, SendError, Transport,
};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use mio::Evented;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
//use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct HttpConnection {
    endpoint: String,
}

impl Connection for HttpConnection {
    fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
        unimplemented!()
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        unimplemented!()
    }

    fn remote_endpoint(&self) -> String {
        unimplemented!()
    }

    fn local_endpoint(&self) -> String {
        // FIXME - this won't always be self.endpoint, might be a firewall endpoint
        self.endpoint.clone()
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        unimplemented!()
    }

    fn evented(&self) -> &dyn Evented {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct HttpTransport {}

impl Transport for HttpTransport {
    fn accepts(&self, address: &str) -> bool {
        address.starts_with("http://")
    }

    fn connect(&mut self, _endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        /*
        if !self.accepts(endpoint) {
            return Err(ConnectError::ProtocolError(format!(
                "Invalid protocol \"{}\"",
                endpoint
            )));
        }
        */

        Ok(Box::new(HttpConnection {
            endpoint: "foo".to_string(),
        }))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        match HttpListener::new(bind) {
            Ok(listener) => Ok(Box::new(listener)),
            Err(err) => Err(err),
        }
    }
}

pub struct HttpListener {
    handle: thread::JoinHandle<()>,
    rx: Receiver<()>,
}

impl HttpListener {
    fn new(bind: &str) -> Result<HttpListener, ListenError> {
        let addr = ([127, 0, 0, 1], 18080).into();
        let service = make_service_fn(|_conn| async {
            Ok::<_, hyper::Error>(service_fn(handle_request))
        });
        let (tx, rx) = channel();

        let handle = thread::spawn(move || {
            let mut rt = tokio::runtime::Builder::new()
                .basic_scheduler()
                .build().unwrap();

            rt.block_on(async {
                let server = Server::bind(&addr).serve(service);
                tx.send(()).unwrap();
                let _ = server.await.unwrap();
            })
        });


        Ok(HttpListener {
            handle,
            rx,
        })
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

impl Listener for HttpListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        self.rx.recv().unwrap();
        unimplemented!();
    }

    fn endpoint(&self) -> String {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transport::tests;

    use mio::Ready;
    use std::sync::mpsc::channel;
    use std::thread;

    #[test]
    fn test_accepts() {
        let transport = HttpTransport::default();
        assert!(transport.accepts("http://127.0.0.1:0"));
        assert!(transport.accepts("http://somewhere.example.com:8080"));

        assert!(!transport.accepts("https://somewhere.example.com:8443"));
    }

    #[test]
    fn test_listen() {
        let server_url = "http://localhost:8080";
        let client_url = server_url.clone();

        // A channel used for the purposes of controlling the timing of the client thread.
        let (client_tx, client_rx) = channel();

        // A channel used for the purposes of controlling the timing of the server thread.
        let (server_tx, server_rx) = channel();

        // Setup a client thread to communicate with the bound port.
        let client_handle = thread::spawn(move || {
            client_rx.recv().unwrap();
        });

        // Setup a server thread to communicate with the bound port.
        let server_handle = thread::spawn(move || {
            server_rx.recv().unwrap();

            // Server Step 1
            //
            // Setup the transport and then accept() for a connection.
            let mut transport = HttpTransport::default();
            let mut listener = transport.listen(server_url).unwrap();
            let connection = listener.accept();
        });

        // Proceed to Server Step 1
        server_tx.send(()).unwrap();

        // Proceed to Client Step 1
        client_tx.send(()).unwrap();

        // Join all threads
        client_handle.join().unwrap();
        server_handle.join().unwrap();
    }

    #[test]
    fn test_transport() {
        let transport = HttpTransport::default();

        tests::test_transport(transport, "http://127.0.0.1:0");
    }

    #[test]
    fn test_poll() {
        let transport = HttpTransport::default();
        tests::test_poll(
            transport,
            "127.0.0.1:0",
            Ready::readable() | Ready::writable(),
        );
    }
}
