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

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::rest_api::{into_bytes, ErrorResponse, Method, Resource};
use super::super::sessions::{Claims, validate_token};

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
    // rest_config: Arc<BiomeRestConfig>,
    // token_issuer: Arc<AccessTokenIssuer>,
) -> Resource {
    Resource::build("/biome/users/{user_id}/keys").add_method(Method::Post, move |request, payload| {
        // let credentials_store = credentials_store.clone();
        // let rest_config = rest_config.clone();
        // let token_issuer = token_issuer.clone();
        let f = request.headers().get("Authorization").unwrap();
        Box::new(HttpResponse::Ok().json(format!("auto {:?}", f)).into_future())
    })
}
