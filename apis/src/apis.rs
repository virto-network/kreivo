mod assets;

pub use assets::AssetsAPI;

/// A set of APIs to interact between applications (like Smart Contracts) and
/// the Kreivo runtime.
pub trait KreivoAPI<Env> {
	/// Manipulation of arbitrary assets.
	type Assets: AssetsAPI<Env>;

	fn assets(&self) -> &Self::Assets;
}
