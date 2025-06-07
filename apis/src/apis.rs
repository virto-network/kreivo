mod assets;
mod error;
mod listings;
mod memberships;

pub use assets::*;
pub use error::*;
pub use listings::*;
pub use memberships::*;

/// A set of APIs to interact between applications (like Smart Contracts) and
/// the Kreivo runtime.
pub trait KreivoAPI<Ext> {
	/// Manipulation of arbitrary assets.
	type Assets: AssetsAPI<Ext>;
	/// Handling of listings for a merchant: inventories and items.
	type Listings: ListingsInventoriesAPI<Ext> + ListingsItemsAPI<Ext>;
	/// Management of group's memberships
	type Memberships: MembershipsAPI<Ext>;
}
