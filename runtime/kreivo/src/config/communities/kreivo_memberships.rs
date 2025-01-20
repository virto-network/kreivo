use super::*;

use fc_traits_memberships::OnMembershipAssigned;
use frame_support::traits::nonfungibles_v2::{Inspect as NonFunsInspect, Mutate};

const WELL_KNOWN_ATTR_KEYS: [&[u8]; 3] = [b"membership_member_rank", b"membership_gas", b"membership_expiration"];

parameter_types! {
	pub CopySystemAttributesOnAssign: Box<dyn OnMembershipAssigned<AccountId, CommunityId, MembershipId>> =
		Box::new(|_, group, m| {
			for key in WELL_KNOWN_ATTR_KEYS.into_iter() {
				if let Some(value) = CommunityMemberships::system_attribute(&group, Some(&m), key) {
					<CommunityMemberships as Mutate<_, _>>::set_attribute(&group, &m, key, &value)?;
				}
			}

			Ok(())
		});
}
