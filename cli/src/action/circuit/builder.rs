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

use splinter::admin::messages::*;

#[derive(Clone)]
pub struct ServiceBuilder {
    service_id: String,
    service_args: Vec<(String, String)>,
    builder: SplinterServiceBuilder,
}

impl ServiceBuilder {
    pub fn new(service_id: &str, allowed_nodes: &[String]) -> ServiceBuilder {
        let builder = SplinterServiceBuilder::new()
            .with_service_id(service_id)
            .with_allowed_nodes(allowed_nodes);
        ServiceBuilder {
            service_id: service_id.to_string(),
            service_args: vec![],
            builder,
        }
    }

    fn build(self) -> Result<SplinterService, String> {
        self.builder
            .with_arguments(&self.service_args)
            .build()
            .map_err(|err| err.to_string())
    }
}

pub struct MessageBuilder {
    services: Vec<ServiceBuilder>,
    nodes: Vec<SplinterNode>,
}

impl MessageBuilder {
    pub fn apply_service_type(&mut self, service_id_match: &str, service_type: &str) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|mut service_builder| {
                if is_match(service_id_match, &service_builder.service_id) {
                    service_builder.builder =
                        service_builder.builder.with_service_type(service_type);
                }
                service_builder
            })
            .collect();
    }

    pub fn apply_service_arguments(&mut self, service_id_match: &str, args: &(String, String)) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|mut service_builder| {
                if is_match(service_id_match, &service_builder.service_id) {
                    service_builder.service_args.push(args.clone())
                }
                service_builder
            })
            .collect();
    }

    pub fn add_node(&mut self, node_id: &str, endpoint: &str) {
        let node = SplinterNodeBuilder::new()
            .with_node_id(node_id)
            .with_endpoint(endpoint)
            .build()
            .expect("adsas");
        self.nodes.push(node);
    }

    pub fn build(self, management_type: &str) -> Result<CreateCircuit, String> {
        let circuit_id = self.make_circuit_id();

        let services = self
            .services
            .into_iter()
            .try_fold::<_, _, Result<_, String>>(Vec::new(), |mut acc, builder| {
                let service = builder.build()?;
                acc.push(service);
                Ok(acc)
            })?;

        let create_circuit = CreateCircuitMessageBuilder::new()
            .with_circuit_id(&circuit_id)
            .with_members(&self.nodes)
            .with_roster(&services)
            .with_circuit_management_type(management_type)
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
