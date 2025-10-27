use super::*;

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq, TryFromPrimitive)]
pub enum ListingsApiError {
	/// The contract does not have an associated `MerchantId`, therefore it
	/// cannot use the Listing APIs.
	NoMerchantId,
	/// The specified inventory is not known.
	UnknownInventory,
	/// There was an error while trying to create an inventory.
	FailedToCreateInventory,
	/// The inventory is already archived.
	ArchivedInventory,
	/// There was an error while trying to archive an existing inventory.
	FailedToArchiveInventory,
	/// There was an error while trying to publish a new item.
	FailedToPublishItem,
	/// The specified item is not known.
	UnknownItem,
	NotForResale,
	ItemNonTransferable,
	/// It was not possible to modify the `not_for_resale` state on an item.
	FailedToSetNotForResale,
	/// It was not possible to modify the `transferable` state on an item.
	FailedToSetTransferable,
	/// It was not possible to set some attribute.
	FailedToSetAttribute,
	/// Transferring an item is not possible.
	CannotTransfer,
	/// It was not possible to set the metadata.
	FailedToSetMetadata,
}

impl From<ListingsApiError> for KreivoApisError {
	fn from(error: ListingsApiError) -> Self {
		KreivoApisError::Listings(error)
	}
}
