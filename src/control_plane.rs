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
//! let all = keys_api::list_keys_v1_keys_get(cp.config(), None, None).await.unwrap();
//! # let _ = all;
//! # Ok(())
//! # }
//! ```

pub use otari_client::apis;
pub use otari_client::apis::configuration::Configuration;
pub use otari_client::models;

use otari_client::apis::{budgets_api, keys_api, pricing_api, usage_api, users_api, Error};

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
    ) -> Result<models::CreateKeyResponse, Error<keys_api::CreateKeyV1KeysPostError>> {
        keys_api::create_key_v1_keys_post(self.config, create_key_request).await
    }

    pub async fn get(
        &self,
        key_id: &str,
    ) -> Result<models::KeyInfo, Error<keys_api::GetKeyV1KeysKeyIdGetError>> {
        keys_api::get_key_v1_keys_key_id_get(self.config, key_id).await
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::KeyInfo>, Error<keys_api::ListKeysV1KeysGetError>> {
        keys_api::list_keys_v1_keys_get(self.config, skip, limit).await
    }

    pub async fn update(
        &self,
        key_id: &str,
        update_key_request: models::UpdateKeyRequest,
    ) -> Result<models::KeyInfo, Error<keys_api::UpdateKeyV1KeysKeyIdPatchError>> {
        keys_api::update_key_v1_keys_key_id_patch(self.config, key_id, update_key_request).await
    }

    pub async fn delete(
        &self,
        key_id: &str,
    ) -> Result<(), Error<keys_api::DeleteKeyV1KeysKeyIdDeleteError>> {
        keys_api::delete_key_v1_keys_key_id_delete(self.config, key_id).await
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
    ) -> Result<models::UserResponse, Error<users_api::CreateUserV1UsersPostError>> {
        users_api::create_user_v1_users_post(self.config, create_user_request).await
    }

    pub async fn get(
        &self,
        user_id: &str,
    ) -> Result<models::UserResponse, Error<users_api::GetUserV1UsersUserIdGetError>> {
        users_api::get_user_v1_users_user_id_get(self.config, user_id).await
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::UserResponse>, Error<users_api::ListUsersV1UsersGetError>> {
        users_api::list_users_v1_users_get(self.config, skip, limit).await
    }

    pub async fn update(
        &self,
        user_id: &str,
        update_user_request: models::UpdateUserRequest,
    ) -> Result<models::UserResponse, Error<users_api::UpdateUserV1UsersUserIdPatchError>> {
        users_api::update_user_v1_users_user_id_patch(self.config, user_id, update_user_request)
            .await
    }

    pub async fn delete(
        &self,
        user_id: &str,
    ) -> Result<(), Error<users_api::DeleteUserV1UsersUserIdDeleteError>> {
        users_api::delete_user_v1_users_user_id_delete(self.config, user_id).await
    }

    pub async fn get_usage(
        &self,
        user_id: &str,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<
        Vec<models::UsageLogResponse>,
        Error<users_api::GetUserUsageV1UsersUserIdUsageGetError>,
    > {
        users_api::get_user_usage_v1_users_user_id_usage_get(self.config, user_id, skip, limit)
            .await
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
    ) -> Result<models::BudgetResponse, Error<budgets_api::CreateBudgetV1BudgetsPostError>> {
        budgets_api::create_budget_v1_budgets_post(self.config, create_budget_request).await
    }

    pub async fn get(
        &self,
        budget_id: &str,
    ) -> Result<models::BudgetResponse, Error<budgets_api::GetBudgetV1BudgetsBudgetIdGetError>>
    {
        budgets_api::get_budget_v1_budgets_budget_id_get(self.config, budget_id).await
    }

    pub async fn list(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<models::BudgetResponse>, Error<budgets_api::ListBudgetsV1BudgetsGetError>> {
        budgets_api::list_budgets_v1_budgets_get(self.config, skip, limit).await
    }

    pub async fn update(
        &self,
        budget_id: &str,
        update_budget_request: models::UpdateBudgetRequest,
    ) -> Result<models::BudgetResponse, Error<budgets_api::UpdateBudgetV1BudgetsBudgetIdPatchError>>
    {
        budgets_api::update_budget_v1_budgets_budget_id_patch(
            self.config,
            budget_id,
            update_budget_request,
        )
        .await
    }

    pub async fn delete(
        &self,
        budget_id: &str,
    ) -> Result<(), Error<budgets_api::DeleteBudgetV1BudgetsBudgetIdDeleteError>> {
        budgets_api::delete_budget_v1_budgets_budget_id_delete(self.config, budget_id).await
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
    ) -> Result<Vec<models::PricingResponse>, Error<pricing_api::ListPricingV1PricingGetError>>
    {
        pricing_api::list_pricing_v1_pricing_get(self.config, skip, limit).await
    }

    pub async fn get(
        &self,
        model_key: &str,
        as_of: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<models::PricingResponse, Error<pricing_api::GetPricingV1PricingModelKeyGetError>>
    {
        pricing_api::get_pricing_v1_pricing_model_key_get(self.config, model_key, as_of).await
    }

    pub async fn set(
        &self,
        set_pricing_request: models::SetPricingRequest,
    ) -> Result<models::PricingResponse, Error<pricing_api::SetPricingV1PricingPostError>> {
        pricing_api::set_pricing_v1_pricing_post(self.config, set_pricing_request).await
    }

    pub async fn delete(
        &self,
        model_key: &str,
        effective_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<(), Error<pricing_api::DeletePricingV1PricingModelKeyDeleteError>> {
        pricing_api::delete_pricing_v1_pricing_model_key_delete(
            self.config,
            model_key,
            effective_at,
        )
        .await
    }

    pub async fn get_history(
        &self,
        model_key: &str,
    ) -> Result<
        Vec<models::PricingResponse>,
        Error<pricing_api::GetPricingHistoryV1PricingModelKeyHistoryGetError>,
    > {
        pricing_api::get_pricing_history_v1_pricing_model_key_history_get(self.config, model_key)
            .await
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
    ) -> Result<Vec<models::UsageEntry>, Error<usage_api::ListUsageV1UsageGetError>> {
        usage_api::list_usage_v1_usage_get(self.config, start_date, end_date, user_id, skip, limit)
            .await
    }
}
