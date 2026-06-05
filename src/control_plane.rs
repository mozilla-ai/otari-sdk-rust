//! Control-plane (management) client: keys, users, budgets, pricing, usage.
//!
//! Re-exports the generated [`otari_control_plane`] client. These endpoints
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

pub use otari_control_plane::apis;
pub use otari_control_plane::apis::configuration::Configuration;
pub use otari_control_plane::models;
