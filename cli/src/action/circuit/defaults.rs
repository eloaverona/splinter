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
use crate::manager::{DefaultValueManager, DefaultValueManagerError};

use super::Action;

pub struct SetServiceTypeDefaultAction;

impl Action for SetServiceTypeDefaultAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let service_type = match args.value_of("service_type") {
            Some(service) => service,
            None => return Err(CliError::ActionError("service-type is required".into())),
        };

        let default_manager = DefaultValueManager::default();
        default_manager.set_default_service_type(service_type, args.is_present("force"))?;

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

        let default_manager = DefaultValueManager::default();
        default_manager.set_default_management_type(management_type, args.is_present("force"))?;

        Ok(())
    }
}

impl From<DefaultValueManagerError> for CliError {
    fn from(err: DefaultValueManagerError) -> Self {
        CliError::ActionError(format!("Failed to perform defaults operation: {}", err))
    }
}
