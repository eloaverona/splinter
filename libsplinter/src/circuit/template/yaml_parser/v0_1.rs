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

#[derive(Deserialize, Debug, Clone)]
pub struct YamlCircuitCreateTemplate {
    version: String,
    args: Vec<YamlRuleArgument>,
    rules: YamlRules,
}

impl YamlCircuitCreateTemplate {
    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn args(&self) -> Vec<YamlRuleArgument> {
        self.args.clone()
    }

    pub fn rules(&self) -> YamlRules {
        self.rules.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct YamlRuleArgument {
    name: String,
    required: bool,
    #[serde(rename = "default")]
    default_value: Option<String>,
}

impl YamlRuleArgument {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn default_value(&self) -> Option<String> {
        self.default_value.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct YamlRules {
    set_management_type: YamlCircuitManagement,
    create_services: Option<YamlCreateServices>,
}

impl YamlRules {
    pub fn set_management_type(&self) -> YamlCircuitManagement {
        self.set_management_type.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct YamlCircuitManagement {
    management_type: String,
}

impl YamlCircuitManagement {
    pub fn management_type(&self) -> String {
        self.management_type.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct YamlCreateServices {
    service_type: String,
    service_args: Vec<YamlServiceArgument>,
    first_service: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct YamlServiceArgument {
    key: String,
    value: String,
}

impl YamlCreateServices {
    pub fn service_type(&self) -> String {
        self.service_type.clone()
    }

    pub fn service_args(&self) -> Vec<YamlServiceArgument> {
        self.service_args.clone()
    }

    pub fn first_service(&self) -> String {
        self.first_service.clone()
    }
}
