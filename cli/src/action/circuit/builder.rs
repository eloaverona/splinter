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

use reqwest::Url;
use uuid::Uuid;

use super::defaults::{get_default_value_store, MANAGEMENT_TYPE_KEY, SERVICE_TYPE_KEY};
use crate::error::CliError;
use crate::store::default_value::{DefaultValueStore, FileBackedDefaultStore};
use crate::store::node::{FileBackedNodeStore, NodeStore};
use splinter::admin::messages::*;

pub struct MessageBuilder {
    services: Vec<SplinterServiceBuilder>,
    nodes: Vec<SplinterNode>,
    management_type: Option<String>,
    node_store: Box<dyn NodeStore>,
    default_store: Box<dyn DefaultValueStore>,
}

impl MessageBuilder {
    pub fn new() -> MessageBuilder {
        MessageBuilder {
            services: vec![],
            nodes: vec![],
            management_type: None,
            node_store: Box::new(FileBackedNodeStore::default()),
            default_store: Box::new(FileBackedDefaultStore::default()),
        }
    }

    pub fn apply_service_type(&mut self, service_id_match: &str, service_type: &str) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|mut service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                if is_match(service_id_match, &service_id) {
                    service_builder.with_service_type(service_type)
                } else {
                    service_builder
                }
            })
            .collect();
    }

    pub fn apply_service_arguments(&mut self, service_id_match: &str, args: &(String, String)) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|mut service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                if is_match(service_id_match, &service_id) {
                    let mut service_args = service_builder.arguments().unwrap_or_default();
                    service_args.push(args.clone());
                    service_builder.with_arguments(&service_args)
                } else {
                    service_builder
                }
            })
            .collect();
    }

    pub fn add_node(&mut self, node: &str) -> Result<(), String> {
        let (node_id, endpoint) = match self
            .node_store
            .get_node(node)
            .map_err(|err| err.to_string())?
        {
            Some(store_node) => (store_node.alias(), store_node.endpoint()),
            None => (
                make_node_id_from_endpoint(node).expect("Err"),
                node.to_string(),
            ),
        };

        let node = SplinterNodeBuilder::new()
            .with_node_id(&node_id)
            .with_endpoint(&endpoint)
            .build()
            .expect("adsas");
        self.nodes.push(node);
        Ok(())
    }

    pub fn set_management_type(&mut self, management_type: &str) {
        self.management_type = Some(management_type.into())
    }

    pub fn build(self) -> Result<CreateCircuit, String> {
        let circuit_id = self.make_circuit_id();
        let default_store = get_default_value_store();

        let management_type = match self.management_type {
            Some(management_type) => management_type,
            None => {
                match default_store
                    .get_default_value(MANAGEMENT_TYPE_KEY)
                    .expect("err")
                {
                    Some(management_type) => management_type.value(),
                    None => {
                        return Err(
                            "Management type not provided and no default value set".to_string()
                        )
                    }
                }
            }
        };

        let services = self
            .services
            .into_iter()
            .try_fold::<_, _, Result<_, String>>(Vec::new(), |mut acc, mut builder| {
                if builder.service_type().is_none() {
                    //let default_store = get_default_value_store();
                    builder = match default_store
                        .get_default_value(SERVICE_TYPE_KEY)
                        .expect("erer")
                    {
                        Some(service_type) => builder.with_service_type(&service_type.value()),
                        None => {
                            return Err("Service has no service type and no default value is set"
                                .to_string())
                        }
                    }
                }
                let service = builder.build().expect("Errrr");
                acc.push(service);
                Ok(acc)
            })?;

        let create_circuit = CreateCircuitMessageBuilder::new()
            .with_circuit_id(&circuit_id)
            .with_members(&self.nodes)
            .with_roster(&services)
            .with_circuit_management_type(&management_type)
            .build()
            .map_err(|err| err.to_string())?;
        Ok(create_circuit)
    }

    fn make_circuit_id(&self) -> String {
        let partial_circuit_id = self.nodes.iter().fold(String::new(), |mut acc, member| {
            acc.push_str(&format!("::{}", member.node_id));
            acc
        });

        format!("{}::{}", partial_circuit_id, Uuid::new_v4().to_string())
    }

    pub fn add_service(&mut self, service_id: &str, allowed_nodes: &[String]) {
        let service_builder = SplinterServiceBuilder::new()
            .with_service_id(service_id)
            .with_allowed_nodes(allowed_nodes);
        self.services.push(service_builder);
    }
}

fn is_match(service_id_match: &str, service_id: &str) -> bool {
    service_id_match.split("*").fold(true, |is_match, part| {
        if part.len() != service_id_match.len() {
            is_match && service_id.contains(part)
        } else {
            service_id == part
        }
    })
}

fn validate_node_endpont(endpoint: &str) -> Result<(), CliError> {
    if let Err(err) = Url::parse(endpoint) {
        Err(CliError::ActionError(format!(
            "{} is not a valid url: {}",
            endpoint, err
        )))
    } else {
        Ok(())
    }
}
fn make_node_id_from_endpoint(endpoint: &str) -> Result<String, CliError> {
    match Url::parse(endpoint) {
        Ok(url) => {
            let host = match url.host_str() {
                Some(host) => host,
                None => {
                    return Err(CliError::ActionError(format!(
                        "Invalid node endpoint: {}",
                        endpoint
                    )))
                }
            };
            let port = match url.port_or_known_default() {
                Some(port) => port,
                None => {
                    return Err(CliError::ActionError(format!(
                        "Invalid node endpoint: {}",
                        endpoint
                    )))
                }
            };
            Ok(format!("{}_{}", host, port))
        }
        Err(err) => Err(CliError::ActionError(format!(
            "Invalid node endpoint or node alias has not been set: {}",
            endpoint
        ))),
    }
}
