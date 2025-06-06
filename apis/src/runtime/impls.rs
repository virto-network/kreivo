use super::*;

mod assets;
mod listings;
mod memberships;

pub trait ChainExtensionDispatch<E> {
	fn call(&self, ext: &E) -> Result<Vec<u8>, KreivoApisError>;
}
