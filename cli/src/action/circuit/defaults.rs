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

use clap::ArgMatches;

use crate::error::CliError;
use crate::store::default_value::{
    DefaultStoreError, DefaultValue, DefaultValueStore, FileBackedDefaultStore,
};

const MANAGEMENT_TYPE_KEY: &str = "management_type";
const SERVICE_TYPE_KEY: &str = "service_type";

use super::Action;

pub struct SetServiceTypeDefaultAction;

impl Action for SetServiceTypeDefaultAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let service_type = match args.value_of("service_type") {
            Some(service) => service,
            None => return Err(CliError::ActionError("service-type is required".into())),
        };

        let store = get_default_value_store();

        if !args.is_present("force") && store.get_default_value(SERVICE_TYPE_KEY)?.is_some() {
            return Err(CliError::ActionError(format!(
                "Default value for {} is already in use",
                SERVICE_TYPE_KEY
            )));
        }

        let default_value = DefaultValue::new(SERVICE_TYPE_KEY, service_type);
        store.set_default_value(&default_value)?;

        Ok(())
    }
}

pub struct UnsetServiceTypeDefaultAction;

impl Action for UnsetServiceTypeDefaultAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let store = get_default_value_store();
        store.unset_default_value(SERVICE_TYPE_KEY)?;
        Ok(())
    }
}

pub struct GetServiceTypeDefaultAction;

impl Action for GetServiceTypeDefaultAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let store = get_default_value_store();

        match store.get_default_value(SERVICE_TYPE_KEY)? {
            Some(default_value) => println!("{} {}", default_value.key(), default_value.value()),
            None => {
                return Err(CliError::ActionError(format!(
                    "Default value for {} is not set",
                    SERVICE_TYPE_KEY
                )))
            }
        }

        Ok(())
    }
}

pub struct SetManagementTypeDefaultAction;

impl Action for SetManagementTypeDefaultAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let management_type = match args.value_of("management_type") {
            Some(management) => management,
            None => return Err(CliError::ActionError("management-type is required".into())),
        };

        let store = get_default_value_store();

        if !args.is_present("force") && store.get_default_value(MANAGEMENT_TYPE_KEY)?.is_some() {
            return Err(CliError::ActionError(format!(
                "Default value for {} is already in use",
                MANAGEMENT_TYPE_KEY
            )));
        }

        let default_value = DefaultValue::new(MANAGEMENT_TYPE_KEY, management_type);
        store.set_default_value(&default_value)?;

        Ok(())
    }
}

pub struct UnsetManagementTypeDefaultAction;

impl Action for UnsetManagementTypeDefaultAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let store = get_default_value_store();
        store.unset_default_value(MANAGEMENT_TYPE_KEY)?;
        Ok(())
    }
}

pub struct GetManagementTypeDefaultAction;

impl Action for GetManagementTypeDefaultAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let store = get_default_value_store();

        match store.get_default_value(MANAGEMENT_TYPE_KEY)? {
            Some(default_value) => println!("{} {}", default_value.key(), default_value.value()),
            None => {
                return Err(CliError::ActionError(format!(
                    "Default value for {} is not set",
                    MANAGEMENT_TYPE_KEY
                )))
            }
        }

        Ok(())
    }
}

pub struct ListDefaultsAction;

impl Action for ListDefaultsAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let store = get_default_value_store();

        let defaults = store.list_default_values()?;
        if defaults.is_empty() {
            println!("No defaults have been set yet");
        } else {
            defaults.iter().for_each(|default_val| {
                println!("{} {}", default_val.key(), default_val.value());
            })
        }
        Ok(())
    }
}

fn get_default_value_store() -> FileBackedDefaultStore {
    FileBackedDefaultStore::default()
}

impl From<DefaultStoreError> for CliError {
    fn from(err: DefaultStoreError) -> Self {
        CliError::ActionError(format!("Failed to perform defaults operation: {}", err))
    }
}
