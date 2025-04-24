use crate::{
	mock::{self, *},
	Apps, Error, Event,
};
use frame_contrib_traits::listings::item::ItemPrice;
use frame_contrib_traits::listings::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::Mutate};
use pallet_contracts_fixtures::compile_module;
use sp_runtime::DispatchError;

pub const APP_ID: AppId = 0;
pub const LICENSE_ID: LicenseId = 0;

fn contract(name: &'static str) -> Vec<u8> {
	compile_module::<Test>(name).unwrap().0
}

fn code_hash(name: &'static str) -> <Test as frame_system::Config>::Hash {
	compile_module::<Test>(name).unwrap().1
}

mod publish {
	use super::*;

	#[test]
	fn fails_if_bad_origin() {
		mock::new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::publish(RuntimeOrigin::root(), vec![], None, None),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn fails_if_caller_cannot_reserve_storage_deposit() {
		mock::new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::publish(RuntimeOrigin::signed(BOB), contract("call"), None, None),
				pallet_contracts::Error::<Test>::StorageDepositNotEnoughFunds
			);
		})
	}

	#[test]
	fn it_works() {
		mock::new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::publish(
				RuntimeOrigin::signed(ALICE),
				contract("call"),
				None,
				None
			));

			// The app was published.
			System::assert_has_event(
				Event::<Test>::AppPublished {
					id: APP_ID,
					publisher: ALICE,
					max_instances: None,
					price: None,
				}
				.into(),
			);

			// An inventory for storing the licenses was created.
			assert!(Listings::exists(&(0, APP_ID)));
		})
	}
}

fn new_test_ext() -> TestExternalities {
	let mut t = mock::new_test_ext();
	t.execute_with(|| {
		assert_ok!(ContractStore::publish(
			RuntimeOrigin::signed(ALICE),
			contract("call"),
			None,
			None
		));
	});
	t
}

mod set_parameters {
	use super::*;
	use crate::AppInfo;
	use frame_support::assert_storage_noop;

	#[test]
	fn fails_if_bad_origin() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::set_parameters(RuntimeOrigin::root(), APP_ID, None, None),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn fails_if_app_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::set_parameters(RuntimeOrigin::signed(ALICE), 1, None, None),
				Error::<Test>::AppNotFound
			);
		})
	}

	#[test]
	fn fails_if_not_the_publisher() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::set_parameters(RuntimeOrigin::signed(BOB), APP_ID, None, None),
				Error::<Test>::NoPermission
			);
		})
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_storage_noop!({
				let _ = ContractStore::set_parameters(RuntimeOrigin::signed(ALICE), APP_ID, None, None);
			});
		});

		new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::set_parameters(
				RuntimeOrigin::signed(ALICE),
				APP_ID,
				Some(10),
				None
			));

			assert!(matches!(
				Apps::<Test>::get(0),
				Some(AppInfo {
					max_instances: Some(10),
					..
				})
			));
		});

		new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::set_parameters(
				RuntimeOrigin::signed(ALICE),
				APP_ID,
				None,
				Some(ItemPrice { asset: 0, amount: 10 })
			));

			System::assert_has_event(
				Event::<Test>::AppPriceUpdated {
					id: APP_ID,
					price: ItemPrice { asset: 0, amount: 10 },
				}
				.into(),
			)
		})
	}
}

mod publish_upgrade {
	use super::*;

	#[test]
	fn fails_if_bad_origin() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::publish_upgrade(RuntimeOrigin::root(), APP_ID, contract("balance")),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn fails_if_app_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::publish_upgrade(RuntimeOrigin::signed(ALICE), 1, contract("balance")),
				Error::<Test>::AppNotFound
			);
		})
	}

	#[test]
	fn fails_if_caller_cannot_reserve_storage_deposit() {
		mock::new_test_ext().execute_with(|| {
			Balances::set_balance(&BOB, Balance::MAX / 2);
			assert_ok!(ContractStore::publish(
				RuntimeOrigin::signed(BOB),
				contract("call"),
				None,
				None
			));
			Balances::set_balance(&BOB, 0);
			assert_noop!(
				ContractStore::publish_upgrade(RuntimeOrigin::signed(BOB), APP_ID, contract("balance")),
				pallet_contracts::Error::<Test>::StorageDepositNotEnoughFunds,
			);
		})
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::publish_upgrade(
				RuntimeOrigin::signed(ALICE),
				APP_ID,
				contract("balance")
			));
		})
	}
}

mod request_license {
	use super::*;

	#[test]
	fn fails_if_bad_origin() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::request_license(RuntimeOrigin::root(), APP_ID),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn fails_if_app_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::request_license(RuntimeOrigin::signed(ALICE), 1),
				Error::<Test>::AppNotFound
			);
		})
	}

	#[test]
	fn fails_if_max_licenses_exceeded() {
		mock::new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::publish(
				RuntimeOrigin::signed(ALICE),
				contract("call"),
				Some(0),
				None
			));

			assert_noop!(
				ContractStore::request_license(RuntimeOrigin::signed(BOB), APP_ID),
				Error::<Test>::MaxLicensesExceeded
			);
		})
	}

	#[test]
	fn it_works() {
		// App is free.
		new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::request_license(RuntimeOrigin::signed(BOB), APP_ID));
			assert!(matches!(
				Listings::item(&(0, APP_ID), &LICENSE_ID),
				Some(item::Item { owner: BOB, .. })
			));
		});

		// App has a price.
		mock::new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::publish(
				RuntimeOrigin::signed(ALICE),
				contract("call"),
				None,
				Some(ItemPrice { asset: 0, amount: 10 })
			));
			assert_ok!(ContractStore::request_license(RuntimeOrigin::signed(BOB), APP_ID));

			System::assert_has_event(
				Event::<Test>::AppLicenseEmitted {
					app_id: APP_ID,
					license_id: LICENSE_ID,
				}
				.into(),
			);

			assert!(matches!(
				Listings::item(&(0, 0), &0),
				Some(item::Item {
					owner: ALICE,
					price: Some(ItemPrice { asset: 0, amount: 10 }),
					..
				})
			));
		})
	}
}

fn test_ext_post_license() -> TestExternalities {
	let mut t = new_test_ext();
	t.execute_with(|| {
		assert_ok!(ContractStore::request_license(RuntimeOrigin::signed(BOB), APP_ID));
	});
	t
}

mod instantiate {
	use super::*;
	use pallet_contracts::AddressGenerator;
	use sp_runtime::TokenError;

	#[test]
	fn fails_if_bad_origin() {
		test_ext_post_license().execute_with(|| {
			assert_noop!(
				ContractStore::instantiate(RuntimeOrigin::root(), APP_ID, LICENSE_ID, 0, vec![], vec![]),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn fails_if_instantiation_data_not_found() {
		test_ext_post_license().execute_with(|| {
			assert_noop!(
				ContractStore::instantiate(RuntimeOrigin::signed(ALICE), 1, LICENSE_ID, 0, vec![], vec![]),
				Error::<Test>::AppNotFound
			);
		});

		test_ext_post_license().execute_with(|| {
			assert_noop!(
				ContractStore::instantiate(RuntimeOrigin::signed(ALICE), APP_ID, 1, 0, vec![], vec![]),
				Error::<Test>::LicenseNotFound
			);
		})
	}

	#[test]
	fn fails_if_caller_is_not_the_license_owner() {
		test_ext_post_license().execute_with(|| {
			assert_noop!(
				ContractStore::instantiate(RuntimeOrigin::signed(ALICE), APP_ID, LICENSE_ID, 0, vec![], vec![]),
				Error::<Test>::NoPermission
			);
		})
	}

	#[test]
	fn fails_if_caller_cannot_reserve_storage_deposit() {
		test_ext_post_license().execute_with(|| {
			assert_noop!(
				ContractStore::instantiate(RuntimeOrigin::signed(BOB), APP_ID, LICENSE_ID, 0, vec![], vec![]),
				TokenError::FundsUnavailable
			);
		})
	}

	#[test]
	fn it_works() {
		test_ext_post_license().execute_with(|| {
			Balances::set_balance(&BOB, Balance::MAX / 2);
			assert_ok!(ContractStore::instantiate(
				RuntimeOrigin::signed(BOB),
				APP_ID,
				LICENSE_ID,
				0,
				vec![],
				vec![]
			));

			System::assert_has_event(
				Event::<Test>::AppInstantiated {
					app_id: APP_ID,
					license_id: LICENSE_ID,
					caller: BOB,
				}
				.into(),
			);

			let contract_address = SimpleAddressGenerator::contract_address(&BOB, &code_hash("call"), &[], &[]);

			assert!(matches!(
				Listings::item(&(0, APP_ID), &LICENSE_ID),
				Some(item::Item { owner, .. }) if owner == contract_address
			));
		})
	}
}

mod upgrade {
	use super::*;

	fn new_test_ext() -> TestExternalities {
		let mut t = test_ext_post_license();
		t.execute_with(|| {
			Balances::set_balance(&BOB, Balance::MAX / 2);
			assert_ok!(ContractStore::instantiate(
				RuntimeOrigin::signed(BOB),
				APP_ID,
				LICENSE_ID,
				0,
				vec![],
				vec![]
			));
		});
		t
	}

	#[test]
	fn fails_if_app_instance_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::upgrade(RuntimeOrigin::signed(BOB), 1, LICENSE_ID),
				Error::<Test>::AppNotFound
			);

			assert_noop!(
				ContractStore::upgrade(RuntimeOrigin::signed(BOB), APP_ID, 1),
				Error::<Test>::LicenseNotFound
			);
		})
	}

	#[test]
	fn fails_if_app_is_up_to_date() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				ContractStore::upgrade(RuntimeOrigin::root(), APP_ID, LICENSE_ID),
				Error::<Test>::AppInstanceUpToDate
			);
		})
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(ContractStore::publish_upgrade(
				RuntimeOrigin::signed(ALICE),
				APP_ID,
				contract("balance")
			));

			assert_ok!(ContractStore::upgrade(RuntimeOrigin::root(), APP_ID, LICENSE_ID));
		})
	}
}
