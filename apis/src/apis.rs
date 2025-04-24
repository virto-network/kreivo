mod assets;
mod error;
mod listings;

pub use assets::*;
pub use error::*;
pub use listings::*;

/// A set of APIs to interact between applications (like Smart Contracts) and
/// the Kreivo runtime.
pub trait KreivoAPI<Ext> {
	/// Manipulation of arbitrary assets.
	type Assets: AssetsAPI<Ext>;
	type Listings: ListingsInventoriesAPI<Ext>;
}
