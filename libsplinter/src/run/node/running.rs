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

use std::thread::JoinHandle;
use std::time::Duration;

use crate::admin::client::{AdminServiceClient, ReqwestAdminServiceClient};
use crate::error::InternalError;
use crate::rest_api::actix_web_1::RestApiShutdownHandle;
use crate::rest_api::actix_web_3::RestApi;
use crate::threading::shutdown::ShutdownHandle;

pub(super) enum NodeRestApiVariant {
    ActixWeb1(RestApiShutdownHandle, JoinHandle<()>),
    ActixWeb3(RestApi),
}

pub struct Node {
    pub(super) rest_api_variant: NodeRestApiVariant,
    pub(super) rest_api_port: u16,
}

impl Node {
    pub fn rest_api_port(self: &Node) -> u16 {
        self.rest_api_port
    }

    pub fn admin_service_client(self: &Node) -> Box<dyn AdminServiceClient> {
        Box::new(ReqwestAdminServiceClient::new(
            format!("http://localhost:{}", self.rest_api_port),
            "foo".to_string(),
        ))
    }
}

impl ShutdownHandle for Node {
    fn signal_shutdown(&mut self) {}

    fn wait_for_shutdown(&mut self, _timeout: Duration) -> Result<(), InternalError> {
        Ok(())
    }
}
