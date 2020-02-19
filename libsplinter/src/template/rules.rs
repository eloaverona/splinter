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

use std::collections::HashMap;

use crate::admin::messages::{CreateCircuitBuilder, SplinterServiceBuilder};

use super::{Rule, RuleArgument, RuleError};

const MANAGEMENT_TYPE_RULE_NAME: &str = "set-management-type";
const CREATE_SERVICES_RULE_NAME: &str = "create-services";
const ALL_OTHER_SERVICES: &str = "ALL_OTHER_SERVICES";
const PEER_SERVICES_ARG: &str = "peer_services";

#[derive(Debug)]
pub(super) struct CircuitManagementTypeRule {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CircuitManagement {
    management_type: String,
}

impl Default for CircuitManagementTypeRule {
    fn default() -> Self {
        CircuitManagementTypeRule {
            name: MANAGEMENT_TYPE_RULE_NAME.to_string(),
        }
    }
}

impl Rule<CreateCircuitBuilder> for CircuitManagementTypeRule {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn apply(
        &mut self,
        values: &[u8],
        _: &HashMap<String, String>,
        builder: CreateCircuitBuilder,
    ) -> Result<CreateCircuitBuilder, RuleError> {
        let circuit_management_type = serde_yaml::from_slice::<CircuitManagement>(values)?;
        Ok(builder.with_circuit_management_type(&circuit_management_type.management_type))
    }

    fn get_arguments(&self) -> Vec<RuleArgument> {
        vec![]
    }
}

struct CreateServicesRule {
    name: String,
    args: Vec<RuleArgument>,
    variables: HashMap<
        String,
        Box<dyn Fn(Vec<SplinterServiceBuilder>) -> Result<Vec<SplinterServiceBuilder>, RuleError>>,
    >,
}

#[derive(Deserialize)]
struct CreateServices {
    service_type: String,
    service_args: Vec<HashMap<String, String>>,
    first_service: String,
}

impl Default for CreateServicesRule {
    fn default() -> Self {
        let nodes_arg = RuleArgument {
            name: "NODES".to_string(),
            required: true,
            default_value: None,
        };
        let mut variables = HashMap::new();
        variables.insert(ALL_OTHER_SERVICES.to_string(), all_services());
        CreateServicesRule {
            name: CREATE_SERVICES_RULE_NAME.to_string(),
            args: vec![nodes_arg],
            variables: variables,
        }
    }
}

fn all_services(
) -> Box<dyn Fn(Vec<SplinterServiceBuilder>) -> Result<Vec<SplinterServiceBuilder>, RuleError>> {
    Box::new(|service_builders| {
        let peers = service_builders.iter().try_fold::<_, _, Result<Vec<String>, RuleError>>(Vec::new(), |mut acc, builder| {
            let service_id = builder.service_id()
                .ok_or_else(|| RuleError::InternalError("The service_id must be set before the service argument PEER_SERVICES can be set".to_string()))?;
            acc.push(service_id);
            Ok(acc)
        })?;
        let services = service_builders
            .into_iter()
            .enumerate()
            .map(|(index, builder)| {
                let mut service_peers = peers.clone();
                service_peers.remove(index);
                let mut service_args = builder.arguments().unwrap_or_default();
                service_args.push((PEER_SERVICES_ARG.into(), format!("{:?}", service_peers)));
                builder.with_arguments(&service_args)
            })
            .collect::<Vec<SplinterServiceBuilder>>();
        Ok(services)
    })
}

impl Rule<Vec<SplinterServiceBuilder>> for CreateServicesRule {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn apply(
        &mut self,
        values: &[u8],
        args: &HashMap<String, String>,
        _: Vec<SplinterServiceBuilder>,
    ) -> Result<Vec<SplinterServiceBuilder>, RuleError> {
        let create_services = serde_yaml::from_slice::<CreateServices>(values)
            .expect("failed to parse managment type");
        let nodes = args
            .get("NODES")
            .expect("No nodes")
            .split(",")
            .map(String::from)
            .collect::<Vec<String>>();
        let mut service_id = create_services.first_service;
        let mut service_builders = vec![];
        for node in nodes {
            let mut splinter_service_builder = SplinterServiceBuilder::new()
                .with_service_id(&service_id)
                .with_allowed_nodes(&vec![node])
                .with_service_type(&create_services.service_type);
            //    .with_arguments(arguments: &[(String, String)]);
            service_builders.push(splinter_service_builder);
            service_id = get_next_service_id(&service_id)?;
        }

        for arg in create_services.service_args {
            let key = arg.get("k").ok_or_else(|| {
                RuleError::InvalidFormat(format!("Invalid service_arg: {:?}", arg))
            })?;
            let value = arg.get("v").ok_or_else(|| {
                RuleError::InvalidFormat(format!("Invalid service_arg: {:?}", arg))
            })?;
            if key == "peer-services" && value == ALL_OTHER_SERVICES {
                service_builders =
                    self.variables.get(ALL_OTHER_SERVICES).expect("err")(service_builders)?;
            }
            else {
                service_builders = service_builders.into_iter().map(|builder| {
                    let mut service_args = builder.arguments().unwrap_or_default();
                    service_args.push((key.into(), value.into()));
                    builder.with_arguments(&service_args)
                }).collect::<Vec<SplinterServiceBuilder>>();
            }
        }

        Ok(service_builders)
        // let circuit_management_type = management_type.get("management-type").unwrap();
        // builder.with_circuit_management_type(circuit_management_type)c
    }

    fn get_arguments(&self) -> Vec<RuleArgument> {
        self.args.clone()
    }
}

fn get_next_service_id(current_id: &str) -> Result<String, RuleError> {
    let alphanumeric = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut next_id = current_id.to_string();
    for (char_index, char) in current_id.char_indices().rev() {
        let index = alphanumeric.find(char).ok_or_else(|| {
            RuleError::InvalidFormat(
                "The field first_service must contain only valid base62 characters".to_string(),
            )
        })?;
        match alphanumeric.get(index + 1..index + 2) {
            Some(sub_str) => {
                println!("sub_str {}", sub_str);
                let mut next_id_sub_string = next_id.get(char_index + 1..).unwrap_or_default();
                let new_sub_string = format!("{}{}", sub_str, next_id_sub_string);
                next_id.replace_range(char_index.., &new_sub_string);
                return Ok(next_id);
            }
            None => {
                let mut next_id_sub_string = next_id.get(char_index + 1..).unwrap_or_default();
                let new_sub_string =
                    format!("{}{}", alphanumeric[0..1].to_string(), next_id_sub_string);
                next_id.replace_range(char_index.., &new_sub_string);
            }
        }
    }

    return Err(RuleError::InvalidFormat(
        "Exceed number of services that can be built".to_string(),
    ))?;
}

#[cfg(test)]
mod test {
    use super::*;

    /*
     * Verifies CircuitManagementTypeRule correcly parses the payload and applies it to the
     * circuit_create_builder
     */
    #[test]
    fn test_apply_circuit_management_rule() {
        let management_type_yaml = b"management-type: \"gameroom\"";

        let mut circuit_management_rule = CircuitManagementTypeRule::default();

        let builder = CreateCircuitBuilder::new();

        let result = circuit_management_rule.apply(management_type_yaml, &HashMap::new(), builder);
        assert!(result.is_ok());

        let circuit_create_builder = result.unwrap();
        assert_eq!(
            circuit_create_builder.circuit_management_type(),
            Some("gameroom".to_string())
        );
    }


    /*
     * Verifies CreateServicesRule correcly parses the payload and applies it to the
     * splinter_services_builders
     */
    #[test]
    fn test_apply_create_services_rule() {
        let management_type_yaml = b"management-type: \"gameroom\"";

        let mut circuit_management_rule = CircuitManagementTypeRule::default();

        let builder = CreateCircuitBuilder::new();

        let result = circuit_management_rule.apply(management_type_yaml, &HashMap::new(), builder);
        assert!(result.is_ok());

        let circuit_create_builder = result.unwrap();
        assert_eq!(
            circuit_create_builder.circuit_management_type(),
            Some("gameroom".to_string())
        );
    }
}
