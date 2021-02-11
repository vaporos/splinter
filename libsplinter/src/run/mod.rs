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

mod network;
mod node;

pub use network::Network;
pub use node::Node;
pub use node::RunnableNode;
pub use node::{NodeBuilder, RestApiVariant};

#[cfg(test)]
mod test {
    use crate::threading::shutdown::shutdown;

    use super::*;

    fn single_node_network(rest_api_variant: RestApiVariant) {
        let mut network = Network::new()
            .with_default_rest_api_variant(rest_api_variant)
            .add_nodes_with_defaults(1)
            .unwrap();

        let client = network.node(0).unwrap().admin_service_client();

        // make a call to the port
        let list_slice = client.list_circuits(None).unwrap();

        // do something with list_slice

        shutdown(vec![Box::new(network)]).unwrap();
    }

    #[test]
    fn single_node_network_actix_web_1() {
        single_node_network(RestApiVariant::ActixWeb1);
    }

    #[test]
    fn single_node_network_actix_web_3() {
        single_node_network(RestApiVariant::ActixWeb3);
    }
}
