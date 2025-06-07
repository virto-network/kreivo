use super::*;

use frame_contrib_traits::listings::item::ItemPrice;
use frame_contrib_traits::listings::*;
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;

type InventoryIdOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::InventoryId;
type ItemIdOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::ItemId;

#[derive(Encode, Decode, Clone, DebugNoBound)]
pub enum ListingsApiInfo<T: Config> {
	InventoryExists {
		id: InventoryIdOf<T>,
	},
	InventoryIsActive {
		id: InventoryIdOf<T>,
	},
	InventoryAttribute {
		id: InventoryIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	Create {
		id: InventoryIdOf<T>,
	},
	Archive {
		id: InventoryIdOf<T>,
	},
	InventorySetAttribute {
		id: InventoryIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
		value: BoundedVec<u8, ConstU32<256>>,
	},
	InventoryClearAttribute {
		id: InventoryIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	Item {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemAttribute {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	ItemTransferable {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemCanResell {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	Publish {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		name: BoundedVec<u8, ConstU32<256>>,
		maybe_price: Option<ItemPrice<AssetIdOf<T>, AssetBalanceOf<T>>>,
	},
	SetPrice {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		price: ItemPrice<AssetIdOf<T>, AssetBalanceOf<T>>,
	},
	ClearPrice {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemEnableResell {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemDisableResell {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemEnableTransfer {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemDisableTransfer {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
	},
	ItemSetAttribute {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
		value: BoundedVec<u8, ConstU32<256>>,
	},
	ItemClearAttribute {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	Transfer {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		beneficiary: AccountIdOf<T>,
	},
	CreatorTransfer {
		inventory_id: InventoryIdOf<T>,
		id: ItemIdOf<T>,
		beneficiary: AccountIdOf<T>,
	},
	SetInventoryMetadata {
		inventory_id: InventoryIdOf<T>,
		metadata: BoundedVec<u8, ConstU32<256>>,
	},
	ClearInventoryMetadata {
		inventory_id: InventoryIdOf<T>,
	},
	SetMetadata {
		inventory_id: InventoryIdOf<T>,
		item_id: ItemIdOf<T>,
		metadata: BoundedVec<u8, ConstU32<256>>,
	},
	ClearMetadata {
		inventory_id: InventoryIdOf<T>,
		item_id: ItemIdOf<T>,
	},
}

impl<T, E> TryFrom<&mut Environment<'_, '_, E, BufInBufOutState>> for ListingsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type Error = DispatchError;

	fn try_from(env: &mut Environment<'_, '_, E, BufInBufOutState>) -> Result<Self, Self::Error> {
		match env.func_id() {
			// Inventories
			0x0100 => {
				let id = env.read_as()?;
				Ok(ListingsApiInfo::InventoryExists { id })
			}
			0x0101 => {
				let id = env.read_as()?;
				Ok(ListingsApiInfo::InventoryIsActive { id })
			}
			0x0102 => {
				let (id, key) = env.read_as()?;
				Ok(ListingsApiInfo::InventoryAttribute { id, key })
			}
			0x0103 => {
				let id = env.read_as()?;
				Ok(ListingsApiInfo::Create { id })
			}
			0x0104 => {
				let id = env.read_as()?;
				Ok(ListingsApiInfo::Archive { id })
			}
			0x0105 => {
				let (id, key, value) = env.read_as()?;
				Ok(ListingsApiInfo::InventorySetAttribute { id, key, value })
			}
			0x0106 => {
				let (id, key) = env.read_as()?;
				Ok(ListingsApiInfo::InventoryClearAttribute { id, key })
			}
			0x0107 => {
				let (inventory_id, metadata) = env.read_as()?;
				Ok(ListingsApiInfo::SetInventoryMetadata { inventory_id, metadata })
			}
			0x0108 => {
				let inventory_id = env.read_as()?;
				Ok(ListingsApiInfo::ClearInventoryMetadata { inventory_id })
			}
			// Items
			0x0110 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::Item { inventory_id, id })
			}
			0x0111 => {
				let (inventory_id, id, key) = env.read_as()?;
				Ok(ListingsApiInfo::ItemAttribute { inventory_id, id, key })
			}
			0x0112 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemTransferable { inventory_id, id })
			}
			0x0113 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemCanResell { inventory_id, id })
			}
			0x0114 => {
				let (inventory_id, id, name, maybe_price) = env.read_as()?;
				Ok(ListingsApiInfo::Publish {
					inventory_id,
					id,
					name,
					maybe_price,
				})
			}
			0x0115 => {
				let (inventory_id, id, price) = env.read_as()?;
				Ok(ListingsApiInfo::SetPrice {
					inventory_id,
					id,
					price,
				})
			}
			0x0116 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ClearPrice { inventory_id, id })
			}
			0x0117 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemEnableResell { inventory_id, id })
			}
			0x0118 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemDisableResell { inventory_id, id })
			}
			0x0119 => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemEnableTransfer { inventory_id, id })
			}
			0x011a => {
				let (inventory_id, id) = env.read_as()?;
				Ok(ListingsApiInfo::ItemDisableTransfer { inventory_id, id })
			}
			0x011b => {
				let (inventory_id, id, key, value) = env.read_as()?;
				Ok(ListingsApiInfo::ItemSetAttribute {
					inventory_id,
					id,
					key,
					value,
				})
			}
			0x011c => {
				let (inventory_id, id, key) = env.read_as()?;
				Ok(ListingsApiInfo::ItemClearAttribute { inventory_id, id, key })
			}
			0x011d => {
				let (inventory_id, id, beneficiary) = env.read_as()?;
				Ok(ListingsApiInfo::Transfer {
					inventory_id,
					id,
					beneficiary,
				})
			}
			0x011e => {
				let (inventory_id, id, beneficiary) = env.read_as()?;
				Ok(ListingsApiInfo::CreatorTransfer {
					inventory_id,
					id,
					beneficiary,
				})
			}
			0x011f => {
				let (inventory_id, item_id, metadata) = env.read_as()?;
				Ok(ListingsApiInfo::SetMetadata {
					inventory_id,
					item_id,
					metadata,
				})
			}
			0x0120 => {
				let (inventory_id, item_id) = env.read_as()?;
				Ok(ListingsApiInfo::ClearMetadata { inventory_id, item_id })
			}
			id => {
				log::error!("Called an unregistered `func_id`: {id:}");
				Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
	}
}
