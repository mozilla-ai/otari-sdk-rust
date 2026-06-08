//! Control-plane (management) client: keys, users, budgets, pricing, usage.
//!
//! Re-exports the generated [`otari_client`] typed core. These endpoints
//! authenticate with `Authorization: Bearer <admin/master key>`, distinct from
//! the inference auth. Build a configured client with
//! [`crate::Otari::control_plane`], then call the generated functions:
//!
//! ```no_run
//! # async fn run(client: otari::Otari) -> otari::Result<()> {
//! use otari::control_plane::{apis::keys_api, models::CreateKeyRequest};
//!
//! let cfg = client.control_plane("gateway-master-key");
//! let created = keys_api::create_key_v1_keys_post(&cfg, CreateKeyRequest::new()).await.unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! The management APIs live under [`apis`] (for example
//! [`apis::keys_api`], [`apis::users_api`], [`apis::budgets_api`],
//! [`apis::pricing_api`], [`apis::usage_api`]); their request/response models
//! live under [`models`].

pub use otari_client::apis;
pub use otari_client::apis::configuration::Configuration;
pub use otari_client::models;
