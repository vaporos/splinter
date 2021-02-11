// Copyright 2018-2021 Cargill Incorporated
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

use crate::rest_api::{RestApiBind, RestApiServerError};

use super::{ResourceProvider, RestApi};

pub struct RunnableRestApi {
    pub(super) resource_providers: Vec<Box<dyn ResourceProvider>>,
    pub(super) bind: RestApiBind,
}

impl RunnableRestApi {
    pub fn run(self) -> Result<RestApi, RestApiServerError> {
        let RunnableRestApi {
            resource_providers,
            bind,
        } = self;

        let (bind_url, acceptor_opt) = match bind {
            RestApiBind::Secure {
                bind,
                cert_path,
                key_path,
            } => {
                let mut acceptor =
                    openssl::ssl::SslAcceptor::mozilla_modern(openssl::ssl::SslMethod::tls())?;
                acceptor.set_private_key_file(key_path, openssl::ssl::SslFiletype::PEM)?;
                acceptor.set_certificate_chain_file(&cert_path)?;
                acceptor.check_private_key()?;

                (bind, Some(acceptor))
            }
            RestApiBind::Insecure(url) => (url, None),
        };

        Ok(RestApi::new(bind_url, acceptor_opt, resource_providers)?)
    }
}
