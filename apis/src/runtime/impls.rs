use super::*;

mod assets;
mod listings;

pub trait ChainExtensionDispatch<E> {
	fn call(&self, ext: &E) -> Result<Vec<u8>, KreivoApisError>;
}
