// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Auxiliary struct/enums for parachain runtimes.
//! Taken from polkadot/runtime/common (at a21cd64) and adapted for parachains.

use super::*;
use core::cmp::Ordering;
use frame_support::traits::{Contains, InstanceFilter, PrivilegeCmp};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use sp_runtime::RuntimeDebug;

pub struct RuntimeBlackListedCalls;
impl Contains<RuntimeCall> for RuntimeBlackListedCalls {
	fn contains(call: &RuntimeCall) -> bool {
		!matches!(
			call,
			RuntimeCall::Balances(_)
				| RuntimeCall::Treasury(_)
				| RuntimeCall::Utility(_)
				| RuntimeCall::Assets(_)
				| RuntimeCall::Proxy(_)
		)
	}
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	DecodeWithMemTracking,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	Any,
	/// Can execute any call that does not transfer funds or assets.
	NonTransfer,
	/// Proxy with the ability to reject time-delay proxy announcements.
	CancelProxy,
	/// Assets proxy. Can execute any call from `assets`, **including asset
	/// transfers**.
	Assets,
	/// Owner proxy. Can execute calls related to asset ownership.
	AssetOwner,
	/// Asset manager. Can execute calls related to asset management.
	AssetManager,
	/// Collator selection proxy. Can execute calls related to collator
	/// selection mechanism.
	Collator,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(c, RuntimeCall::Balances { .. } | RuntimeCall::Assets { .. }),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Assets => {
				matches!(
					c,
					RuntimeCall::Assets { .. } | RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. }
				)
			}
			ProxyType::AssetOwner => matches!(
				c,
				RuntimeCall::Assets(KreivoAssetsCall::create { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::start_destroy { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::destroy_accounts { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::destroy_approvals { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::finish_destroy { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::transfer_ownership { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::set_team { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::set_metadata { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::clear_metadata { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::AssetManager => matches!(
				c,
				RuntimeCall::Assets(KreivoAssetsCall::mint { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::burn { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::freeze { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::thaw { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::freeze_asset { .. })
					| RuntimeCall::Assets(KreivoAssetsCall::thaw_asset { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Collator => matches!(
				c,
				RuntimeCall::CollatorSelection { .. } | RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. }
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::Assets, ProxyType::AssetOwner) => true,
			(ProxyType::Assets, ProxyType::AssetManager) => true,
			(ProxyType::NonTransfer, ProxyType::Collator) => true,
			_ => false,
		}
	}
}

/// Used to compare the privilege of an origin inside the scheduler.
pub struct EqualOrGreatestRootCmp;

impl PrivilegeCmp<OriginCaller> for EqualOrGreatestRootCmp {
	fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
		if left == right {
			return Some(Ordering::Equal);
		}
		match (left, right) {
			// Root is greater than anything.
			(OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
			_ => None,
		}
	}
}
