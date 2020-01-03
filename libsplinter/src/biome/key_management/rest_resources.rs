// Copyright 2020 Cargill Incorporated
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

use std::sync::Arc;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::rest_api::{into_bytes, ErrorResponse, Method, Resource};
use super::super::sessions::{Claims, validate_token, TokenValidationError};
use super::super::secrets::SecretManager;
use super::super::rest_api::BiomeRestConfig;




#[derive(Deserialize)]
struct NewKey {
    user_id: String,
    public_key: String,
    encrypted_private_key: String,
    display_name: String
}

/// Defines a REST endpoint for login
pub fn make_key_management_route(
    // credentials_store: Arc<SplinterCredentialsStore>,
    rest_config: Arc<BiomeRestConfig>,
    // token_issuer: Arc<AccessTokenIssuer>,
    secret_manager: Arc<dyn SecretManager>
) -> Resource {
    Resource::build("/biome/users/{user_id}/keys").add_method(Method::Post, move |request, payload| {
        // let credentials_store = credentials_store.clone();
        let rest_config = rest_config.clone();
        // let token_issuer = token_issuer.clone();
        let secret_manager = secret_manager.clone();
        let f = request.headers().get("Authorization").unwrap().to_str().unwrap().split_whitespace().last().unwrap();
        let user_id = request.match_info().get("user_id").unwrap();
        let c = validate_token(f, &secret_manager.secret().unwrap(), &rest_config.issuer(), |claim| {
            if user_id != claim.user_id() {
                return Err(TokenValidationError::InvalidClaim(format!("User is not authorized to add keys for user {}", user_id)));
            }
            Ok(())
        }).expect("err");
        Box::new(HttpResponse::Ok().json(format!("auto {:?}", f)).into_future())
    })
}
