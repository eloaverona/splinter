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

use serde::{Deserialize, Serialize};

use super::super::yaml_store::YamlBackedStore;
use super::{Node, NodeStore, NodeStoreError};

const DEFAULT_FILE_NAME: &str = "node_alias";

#[derive(Serialize, Deserialize)]
pub struct SerdeNode {
    alias: String,
    endpoint: String,
}

impl Into<Node> for SerdeNode {
    fn into(self) -> Node {
        Node {
            alias: self.alias,
            endpoint: self.endpoint,
        }
    }
}

impl From<&Node> for SerdeNode {
    fn from(node: &Node) -> SerdeNode {
        SerdeNode {
            alias: node.alias.clone(),
            endpoint: node.endpoint.clone(),
        }
    }
}

pub struct FileBackedNodeStore {
    file_name: String,
}

impl Default for FileBackedNodeStore {
    fn default() -> Self {
        FileBackedNodeStore {
            file_name: DEFAULT_FILE_NAME.to_owned(),
        }
    }
}

impl YamlBackedStore<SerdeNode> for FileBackedNodeStore {}

impl NodeStore for FileBackedNodeStore {
    fn get_node(&self, alias: &str) -> Result<Option<Node>, NodeStoreError> {
        let nodes = Self::read_data_from_file(&self.file_name)?;

        let node = nodes.into_iter().find_map(|node| {
            if node.alias == alias {
                return Some(node.into());
            }
            None
        });

        Ok(node)
    }

    fn list_nodes(&self) -> Result<Vec<Node>, NodeStoreError> {
        let serde_nodes = Self::read_data_from_file(&self.file_name)?;
        let nodes = serde_nodes
            .into_iter()
            .map(|node| node.into())
            .collect::<Vec<Node>>();
        Ok(nodes)
    }

    fn add_node(&self, new_node: &Node) -> Result<(), NodeStoreError> {
        let mut serde_nodes = Self::read_data_from_file(&self.file_name)?;
        let existing_node_index = serde_nodes.iter().enumerate().find_map(|(index, node)| {
            if new_node.alias() == node.alias {
                Some(index)
            } else {
                None
            }
        });
        if let Some(index) = existing_node_index {
            serde_nodes.remove(index);
        }
        serde_nodes.push(SerdeNode::from(new_node));

        Self::write_to_file(&self.file_name, &serde_nodes)?;

        Ok(())
    }

    fn delete_node(&self, alias: &str) -> Result<(), NodeStoreError> {
        let mut serde_nodes = Self::read_data_from_file(&self.file_name)?;
        let existing_node_index = serde_nodes.iter().enumerate().find_map(|(index, node)| {
            if node.alias == alias {
                Some(index)
            } else {
                None
            }
        });

        match existing_node_index {
            Some(index) => {
                serde_nodes.remove(index);
                Self::write_to_file(&self.file_name, &serde_nodes)?;
            }
            None => {
                return Err(NodeStoreError::NotFound(format!(
                    "Node with alias {} was not found",
                    alias
                )))
            }
        }

        Ok(())
    }
}
