//! Control-plane (management) client: keys, users, budgets, pricing, usage.
//!
//! These endpoints authenticate with `Authorization: Bearer <admin/master
//! key>`, distinct from the inference auth. Build a client with
//! [`crate::Otari::control_plane`], then call the ergonomic aliases on each
//! resource accessor (`keys`, `users`, `budgets`, `pricing`, `usage`):
//!
//! ```no_run
//! # async fn run(client: otari::Otari) -> otari::Result<()> {
//! use otari::control_plane::models::CreateKeyRequest;
//!
//! let cp = client.control_plane("gateway-master-key");
//! let created = cp.keys().create(CreateKeyRequest::new()).await.unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! The generated typed core stays reachable as an escape hatch: pass
//! [`ControlPlane::config`] to the generated functions under [`apis`] (for
//! example [`apis::keys_api`], [`apis::users_api`], [`apis::budgets_api`],
//! [`apis::pricing_api`], [`apis::usage_api`]); their request/response models
//! live under [`models`].
//!
//! ```no_run
//! # async fn run(client: otari::Otari) -> otari::Result<()> {
//! use otari::control_plane::apis::keys_api;
//!
//! let cp = client.control_plane("gateway-master-key");
//! let all = keys_api::keys_list_keys(cp.config(), None, None, None).await.unwrap();
//! # let _ = all;
//! # Ok(())
//! # }
//! ```

pub use crate::_client::apis;
pub use crate::_client::apis::configuration::Configuration;
pub use crate::_client::models;

use crate::_client::apis::{budgets_api, keys_api, pricing_api, usage_api, users_api};
use crate::core::map_error;
use crate::error::Result;

/// Ergonomic control-plane client wrapping a configured [`Configuration`].
///
/// Each accessor (`keys`, `users`, `budgets`, `pricing`, `usage`) returns a
/// resource exposing short, generator-name-free aliases that delegate to the
/// generated functions under [`apis`]. The underlying [`Configuration`] stays
/// reachable via [`ControlPlane::config`] as an escape hatch.
pub struct ControlPlane {
    config: Configuration,
}

impl ControlPlane {
    /// Wrap a configured control-plane [`Configuration`].
    pub fn new(config: Configuration) -> Self {
        Self { config }
    }

    /// The underlying generated configuration (escape hatch): pass it to the
    /// generated functions under [`apis`] for the full generated surface.
    pub fn config(&self) -> &Configuration {
        &self.config
    }

    /// API-keys management endpoints.
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            config: &self.config,
        }
    }

    /// Users management endpoints.
    pub fn users(&self) -> Users<'_> {
        Users {
            config: &self.config,
        }
    }

    /// Budgets management endpoints.
    pub fn budgets(&self) -> Budgets<'_> {
        Budgets {
            config: &self.config,
        }
    }

    /// Model-pricing management endpoints.
    pub fn pricing(&self) -> Pricing<'_> {
        Pricing {
            config: &self.config,
        }
    }

    /// Usage-log management endpoints.
    pub fn usage(&self) -> Usage<'_> {
        Usage {
            config: &self.config,
        }
    }
}

/// Ergonomic aliases for the API-keys management endpoints.
pub struct Keys<'a> {
    config: &'a Configuration,
}

impl Keys<'_> {
    pub async fn create(
        &self,
        create_key_request: models::CreateKeyRequest,
    ) -> Result<models::CreateKeyResponse> {
        keys_api::keys_create_key(self.config, create_key_request)
            .await
            .map_err(map_error)
    }

    pub async fn get(&self, key_id: &str) -> Result<models::KeyInfo> {
        keys_api::keys_get_key(self.config, key_id)
            .await
            .map_err(map_error)
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::KeyInfo>> {
        // workspace_id: a scoping filter the generated core gained that this
        // alias does not surface yet.
        keys_api::keys_list_keys(self.config, skip, limit, None)
            .await
            .map_err(map_error)
    }

    pub async fn update(
        &self,
        key_id: &str,
        update_key_request: models::UpdateKeyRequest,
    ) -> Result<models::KeyInfo> {
        keys_api::keys_update_key(self.config, key_id, update_key_request)
            .await
            .map_err(map_error)
    }

    pub async fn delete(&self, key_id: &str) -> Result<()> {
        keys_api::keys_delete_key(self.config, key_id)
            .await
            .map_err(map_error)
    }
}

/// Ergonomic aliases for the users management endpoints.
pub struct Users<'a> {
    config: &'a Configuration,
}

impl Users<'_> {
    pub async fn create(
        &self,
        create_user_request: models::CreateUserRequest,
    ) -> Result<models::UserResponse> {
        users_api::users_create_user(self.config, create_user_request)
            .await
            .map_err(map_error)
    }

    pub async fn get(&self, user_id: &str) -> Result<models::UserResponse> {
        users_api::users_get_user(self.config, user_id)
            .await
            .map_err(map_error)
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::UserResponse>> {
        users_api::users_list_users(self.config, skip, limit)
            .await
            .map_err(map_error)
    }

    pub async fn update(
        &self,
        user_id: &str,
        update_user_request: models::UpdateUserRequest,
    ) -> Result<models::UserResponse> {
        users_api::users_update_user(self.config, user_id, update_user_request)
            .await
            .map_err(map_error)
    }

    pub async fn delete(&self, user_id: &str) -> Result<()> {
        users_api::users_delete_user(self.config, user_id)
            .await
            .map_err(map_error)
    }

    pub async fn get_usage(
        &self,
        user_id: &str,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::UsageLogResponse>> {
        users_api::users_get_user_usage(self.config, user_id, skip, limit)
            .await
            .map_err(map_error)
    }
}

/// Ergonomic aliases for the budgets management endpoints.
pub struct Budgets<'a> {
    config: &'a Configuration,
}

impl Budgets<'_> {
    pub async fn create(
        &self,
        create_budget_request: models::CreateBudgetRequest,
    ) -> Result<models::BudgetResponse> {
        budgets_api::budgets_create_budget(self.config, create_budget_request)
            .await
            .map_err(map_error)
    }

    pub async fn get(&self, budget_id: &str) -> Result<models::BudgetResponse> {
        budgets_api::budgets_get_budget(self.config, budget_id)
            .await
            .map_err(map_error)
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::BudgetResponse>> {
        budgets_api::budgets_list_budgets(self.config, skip, limit)
            .await
            .map_err(map_error)
    }

    pub async fn update(
        &self,
        budget_id: &str,
        update_budget_request: models::UpdateBudgetRequest,
    ) -> Result<models::BudgetResponse> {
        budgets_api::budgets_update_budget(self.config, budget_id, update_budget_request)
            .await
            .map_err(map_error)
    }

    pub async fn delete(&self, budget_id: &str) -> Result<()> {
        budgets_api::budgets_delete_budget(self.config, budget_id)
            .await
            .map_err(map_error)
    }
}

/// Ergonomic aliases for the model-pricing management endpoints.
pub struct Pricing<'a> {
    config: &'a Configuration,
}

impl Pricing<'_> {
    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::PricingResponse>> {
        pricing_api::pricing_list_pricing(self.config, skip, limit)
            .await
            .map_err(map_error)
    }

    pub async fn get(
        &self,
        model_key: &str,
        as_of: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<models::PricingResponse> {
        pricing_api::pricing_get_pricing(self.config, model_key, as_of)
            .await
            .map_err(map_error)
    }

    pub async fn set(
        &self,
        set_pricing_request: models::SetPricingRequest,
    ) -> Result<models::PricingResponse> {
        pricing_api::pricing_set_pricing(self.config, set_pricing_request)
            .await
            .map_err(map_error)
    }

    pub async fn delete(
        &self,
        model_key: &str,
        effective_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<()> {
        pricing_api::pricing_delete_pricing(self.config, model_key, effective_at)
            .await
            .map_err(map_error)
    }

    pub async fn get_history(&self, model_key: &str) -> Result<Vec<models::PricingResponse>> {
        pricing_api::pricing_get_pricing_history(self.config, model_key)
            .await
            .map_err(map_error)
    }
}

/// Ergonomic aliases for the usage-log management endpoints.
pub struct Usage<'a> {
    config: &'a Configuration,
}

impl Usage<'_> {
    pub async fn list(
        &self,
        start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
        end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
        user_id: Option<&str>,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::UsageEntry>> {
        usage_api::usage_list_usage(
            self.config,
            start_date,
            end_date,
            // user_id is repeatable upstream (user_id=a&user_id=b, max 50). This
            // alias keeps its single-user signature and wraps; multi-user
            // filtering is reachable through the generated core.
            user_id.map(|u| vec![u.to_string()]),
            // status, status_code, model, endpoint, provider, source, source_label,
            // api_key_id, priced, tool, counts_toward_budget, request_group_id,
            // workspace_id: filters the generated core gained that this alias
            // does not surface yet.
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            skip,
            limit,
        )
        .await
        .map_err(map_error)
    }
}
