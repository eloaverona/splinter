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

use uuid::Uuid;

use super::super::node::get_node_store;
use super::defaults::{get_default_value_store, MANAGEMENT_TYPE_KEY, SERVICE_TYPE_KEY};

use crate::error::CliError;
use crate::store::default_value::DefaultValueStore;
use crate::store::node::NodeStore;
use splinter::admin::messages::{
    AuthorizationType, CreateCircuit, CreateCircuitBuilder, SplinterNode, SplinterNodeBuilder,
    SplinterServiceBuilder,
};

pub struct CreateCircuitMessageBuilder {
    services: Vec<SplinterServiceBuilder>,
    nodes: Vec<SplinterNode>,
    management_type: Option<String>,
    authorization_type: Option<AuthorizationType>,
    application_metadata: Vec<u8>,
}

impl CreateCircuitMessageBuilder {
    pub fn new() -> CreateCircuitMessageBuilder {
        CreateCircuitMessageBuilder {
            services: vec![],
            nodes: vec![],
            management_type: None,
            authorization_type: None,
            application_metadata: vec![],
        }
    }

    pub fn apply_service_type(&mut self, service_id_match: &str, service_type: &str) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|service_builder| {
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
            .map(|service_builder| {
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

    pub fn add_node(
        &mut self,
        node_id: &str,
        node_endpoint: Option<String>,
    ) -> Result<(), CliError> {
        let node = make_splinter_node(node_id, node_endpoint)?;
        self.nodes.push(node);
        Ok(())
    }

    pub fn set_management_type(&mut self, management_type: &str) {
        self.management_type = Some(management_type.into());
    }

    pub fn set_authorization_type(&mut self, authorization_type: &str) -> Result<(), CliError> {
        let auth_type = match authorization_type {
            "trust" => AuthorizationType::Trust,
            _ => {
                return Err(CliError::ActionError(format!(
                    "Invalid authorization type {}",
                    authorization_type
                )))
            }
        };

        self.authorization_type = Some(auth_type);
        Ok(())
    }

    pub fn set_application_metadata(&mut self, application_metadata: &[u8]) {
        self.application_metadata = application_metadata.into();
    }

    pub fn build(self) -> Result<CreateCircuit, CliError> {
        let circuit_id = make_circuit_id();
        let default_store = get_default_value_store();

        // if management type is not set check for default value
        let management_type = match self.management_type {
            Some(management_type) => management_type,
            None => match default_store.get_default_value(MANAGEMENT_TYPE_KEY)? {
                Some(management_type) => management_type.value(),
                None => {
                    return Err(CliError::ActionError(
                        "Management type not provided and no default value set".to_string(),
                    ))
                }
            },
        };

        let mut nodes = self.nodes.clone();

        let services =
            self.services
                .into_iter()
                .try_fold(Vec::new(), |mut services, mut builder| {
                    // Check for any allowed nodes that have not been explicitly added to the
                    // circuit definition yet
                    for node_id in builder.allowed_nodes().unwrap_or_default().iter() {
                        if nodes.iter().find(|node| &node.node_id == node_id).is_none() {
                            let node = make_splinter_node(node_id, None)?;
                            nodes.push(node)
                        }
                    }

                    // if service type is not set, check for default value
                    if builder.service_type().is_none() {
                        builder = match default_store.get_default_value(SERVICE_TYPE_KEY)? {
                            Some(service_type) => builder.with_service_type(&service_type.value()),
                            None => {
                                return Err(CliError::ActionError(
                                    "Service has no service type and no default value is set"
                                        .to_string(),
                                ))
                            }
                        }
                    }

                    let service = builder.build().map_err(|err| {
                        CliError::ActionError(format!("Failed to build service: {}", err))
                    })?;
                    services.push(service);
                    Ok(services)
                })?;

        let mut create_circuit_builder = CreateCircuitBuilder::new()
            .with_circuit_id(&circuit_id)
            .with_members(&nodes)
            .with_roster(&services)
            .with_application_metadata(&self.application_metadata)
            .with_circuit_management_type(&management_type);

        if let Some(authorization_type) = self.authorization_type {
            create_circuit_builder =
                create_circuit_builder.with_authorization_type(&authorization_type);
        }

        let create_circuit = create_circuit_builder.build().map_err(|err| {
            CliError::ActionError(format!("Failed to build CreateCircuit message: {}", err))
        })?;
        Ok(create_circuit)
    }

    pub fn add_service(&mut self, service_id: &str, allowed_nodes: &[String]) {
        let service_builder = SplinterServiceBuilder::new()
            .with_service_id(service_id)
            .with_allowed_nodes(allowed_nodes);
        self.services.push(service_builder);
    }
}

fn is_match(service_id_match: &str, service_id: &str) -> bool {
    service_id_match.split('*').fold(true, |is_match, part| {
        if part.len() != service_id_match.len() {
            is_match && service_id.contains(part)
        } else {
            service_id == part
        }
    })
}

fn make_splinter_node(
    node_id: &str,
    node_endpoint: Option<String>,
) -> Result<SplinterNode, CliError> {
    let node_store = get_node_store();
    let endpoint = match node_endpoint {
        Some(endpoint) => endpoint,
        None => match node_store.get_node(node_id)? {
            Some(node) => node.endpoint(),
            None => {
                return Err(CliError::ActionError(format!(
                    "No endpoint provided and an alias for node {} has not been set",
                    node_id
                )))
            }
        },
    };

    let node = SplinterNodeBuilder::new()
        .with_node_id(&node_id)
        .with_endpoint(&endpoint)
        .build()
        .map_err(|err| CliError::ActionError(format!("Failed to build SplinterNode: {}", err)))?;
    Ok(node)
}

fn make_circuit_id() -> String {
    Uuid::new_v4().to_string()
}
