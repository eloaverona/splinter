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

use super::super::{yaml_parser::v1, CircuitTemplateError, CreateCircuitBuilder};
use super::{get_argument_value, is_arg, RuleArgument, Value};

pub(super) struct SetMetadata {
    metadata: Metadata,
}

impl SetMetadata {
    pub fn apply_rule(
        &self,
        builder: CreateCircuitBuilder,
        template_arguments: &[RuleArgument],
    ) -> Result<CreateCircuitBuilder, CircuitTemplateError> {
        match &self.metadata {
            Metadata::Json { metadata } => {
                let mut metadata = metadata
                    .iter()
                    .try_fold::<_, _, Result<_, CircuitTemplateError>>(
                        "{[".to_string(),
                        |mut acc, metadata| {
                            match &metadata.value {
                                Value::Single(value) => {
                                    let value = if is_arg(&value) {
                                        get_argument_value(&value, &template_arguments)?
                                    } else {
                                        value.to_string()
                                    };
                                    acc.push_str(&format!("\"{}\":\"{}\",", metadata.key, value));
                                }
                                Value::List(values) => {
                                    let processed_values = values.iter().try_fold::<_, _, Result<
                                        _,
                                        CircuitTemplateError,
                                    >>(
                                        Vec::new(),
                                        |mut acc, value| {
                                            let value = if is_arg(&value) {
                                                get_argument_value(&value, &template_arguments)?
                                            } else {
                                                value.to_string()
                                            };
                                            acc.push(value);
                                            Ok(acc)
                                        },
                                    )?;
                                    acc.push_str(&format!(
                                        "\"{}\":{:?},",
                                        metadata.key, processed_values
                                    ));
                                }
                            }

                            Ok(acc)
                        },
                    )?;

                metadata.pop();
                metadata.push_str("]}");

                Ok(builder.with_application_metadata(&metadata.as_bytes()))
            }
        }
    }
}

impl From<v1::SetMetadata> for SetMetadata {
    fn from(set_metadata: v1::SetMetadata) -> Self {
        SetMetadata {
            metadata: Metadata::from(set_metadata.metadata().clone()),
        }
    }
}

pub(super) enum Metadata {
    Json { metadata: Vec<JsonMetadata> },
}

impl From<v1::Metadata> for Metadata {
    fn from(metadata: v1::Metadata) -> Self {
        match metadata {
            v1::Metadata::Json { metadata } => Metadata::Json {
                metadata: metadata.into_iter().map(JsonMetadata::from).collect(),
            },
        }
    }
}

pub(super) struct JsonMetadata {
    key: String,
    value: Value,
}

impl From<v1::JsonMetadata> for JsonMetadata {
    fn from(metadata: v1::JsonMetadata) -> Self {
        JsonMetadata {
            key: metadata.key().to_string(),
            value: Value::from(metadata.value().clone()),
        }
    }
}
