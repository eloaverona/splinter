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

use crate::admin::messages::CreateCircuitBuilder;

use super::{Rule, RuleArgument, RuleError};

const MANAGEMENT_TYPE_RULE_NAME: &str = "set-management-type";

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
}
