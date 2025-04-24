use super::*;

type ListingsAPIOf<T, E> = <RuntimeKreivoAPI<T> as KreivoAPI<E>>::Listings;

impl<T, E> ChainExtensionDispatch<E> for ListingsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	fn call(&self, ext: &E) -> Result<Vec<u8>, KreivoApisError> {
		match self {
			ListingsApiInfo::InventoryExists { id } => Ok(ListingsAPIOf::<T, E>::inventory_exists(ext, id).encode()),
			ListingsApiInfo::InventoryIsActive { id } => {
				Ok(ListingsAPIOf::<T, E>::inventory_is_active(ext, id).encode())
			}
			ListingsApiInfo::InventoryAttribute { id, key } => {
				Ok(ListingsAPIOf::<T, E>::inventory_attribute::<_, Vec<u8>>(ext, id, key).encode())
			}
			ListingsApiInfo::Create { id } => ListingsAPIOf::<T, E>::create(ext, id).map(|v| v.encode()),
			ListingsApiInfo::Archive { id } => ListingsAPIOf::<T, E>::archive(ext, id).map(|v| v.encode()),
			ListingsApiInfo::InventorySetAttribute { id, key, value } => {
				ListingsAPIOf::<T, E>::inventory_set_attribute::<_, Vec<u8>>(ext, id, key, value).map(|v| v.encode())
			}
			ListingsApiInfo::InventoryClearAttribute { id, key } => {
				ListingsAPIOf::<T, E>::inventory_clear_attribute::<_, Vec<u8>>(ext, id, key).map(|v| v.encode())
			}
			ListingsApiInfo::Item { inventory_id, id } => {
				Ok(ListingsAPIOf::<T, E>::item(ext, inventory_id, id).encode())
			}
			ListingsApiInfo::ItemAttribute { inventory_id, id, key } => {
				Ok(ListingsAPIOf::<T, E>::item_attribute::<_, Vec<u8>>(ext, inventory_id, id, key).encode())
			}
			ListingsApiInfo::ItemTransferable { inventory_id, id } => {
				Ok(ListingsAPIOf::<T, E>::item_transferable(ext, inventory_id, id).encode())
			}
			ListingsApiInfo::ItemCanResell { inventory_id, id } => {
				Ok(ListingsAPIOf::<T, E>::item_can_resell(ext, inventory_id, id).encode())
			}
			ListingsApiInfo::Publish {
				inventory_id,
				id,
				name,
				maybe_price,
			} => ListingsAPIOf::<T, E>::publish(ext, inventory_id, id, name.to_vec(), maybe_price.clone())
				.map(|v| v.encode()),
			ListingsApiInfo::SetPrice {
				inventory_id,
				id,
				price,
			} => ListingsAPIOf::<T, E>::set_price(ext, inventory_id, id, price.clone()).map(|v| v.encode()),
			ListingsApiInfo::ClearPrice { inventory_id, id } => {
				ListingsAPIOf::<T, E>::clear_price(ext, inventory_id, id).map(|v| v.encode())
			}
			ListingsApiInfo::ItemEnableResell { inventory_id, id } => {
				ListingsAPIOf::<T, E>::item_enable_resell(ext, inventory_id, id).map(|v| v.encode())
			}
			ListingsApiInfo::ItemDisableResell { inventory_id, id } => {
				ListingsAPIOf::<T, E>::item_disable_resell(ext, inventory_id, id).map(|v| v.encode())
			}
			ListingsApiInfo::ItemEnableTransfer { inventory_id, id } => {
				ListingsAPIOf::<T, E>::item_enable_transfer(ext, inventory_id, id).map(|v| v.encode())
			}
			ListingsApiInfo::ItemDisableTransfer { inventory_id, id } => {
				ListingsAPIOf::<T, E>::item_disable_transfer(ext, inventory_id, id).map(|v| v.encode())
			}
			ListingsApiInfo::ItemSetAttribute {
				inventory_id,
				id,
				key,
				value,
			} => ListingsAPIOf::<T, E>::item_set_attribute(ext, inventory_id, id, key, value).map(|v| v.encode()),
			ListingsApiInfo::ItemClearAttribute { inventory_id, id, key } => {
				ListingsAPIOf::<T, E>::item_clear_attribute(ext, inventory_id, id, key).map(|v| v.encode())
			}
		}
	}
}
