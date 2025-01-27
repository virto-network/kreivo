#![cfg_attr(not(any(test, feature = "std")), no_std)]

//! # Kreivo APIs
//!
//! This is a set of APIs that can be used in an application context (like Smart
//! Contracts) to interact with Kreivo.
//!
//! ## APIs
//!
//! Currently available APIs include:
//!
//! - **[`AssetsAPI`][apis::AssetsAPI]:** These APIs can facilitate transactions
//!   regarding assets.

pub mod apis;
#[cfg(feature = "contract")]
mod contract;
#[cfg(feature = "runtime")]
mod runtime;

#[cfg(feature = "contract")]
pub use contract::KreivoApiEnvironment;
#[cfg(feature = "runtime")]
pub use runtime::KreivoChainExtensions;
