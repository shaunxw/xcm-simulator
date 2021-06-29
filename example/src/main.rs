fn main() {}

mod para;
mod relay;

use frame_support::traits::GenesisBuild;
use sp_runtime::AccountId32;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);

decl_test_parachain! {
	pub struct ParaA {
		Runtime = para::Runtime,
		new_ext = para_ext(1),
	}
}

decl_test_parachain! {
	pub struct ParaB {
		Runtime = para::Runtime,
		new_ext = para_ext(2),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay::Runtime,
		XcmConfig = relay::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1, ParaA),
			(2, ParaB),
		],
	}
}

pub const INITIAL_BALANCE: u128 = 1_000_000_000;

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use para::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let parachain_info_config = parachain_info::GenesisConfig {
		parachain_id: para_id.into(),
	};

	<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use relay::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<para::Runtime>;

#[cfg(test)]
mod tests {
	use super::*;

	use codec::Encode;
	use frame_support::assert_ok;
	use xcm::v0::{
		Junction::{self, Parachain, Parent},
		MultiAsset::*,
		MultiLocation::*,
		NetworkId, OriginKind,
		Xcm::*,
	};
	use xcm_simulator::TestExt;

	fn print_events<T: frame_system::Config>(context: &str) {
		println!("------ {:?} events ------", context);
		frame_system::Pallet::<T>::events().iter().for_each(|r| {
			println!("{:?}", r.event);
		});
	}

	#[test]
	fn reserve_transfer() {
		MockNet::reset();

		Relay::execute_with(|| {
			assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
				relay::Origin::signed(ALICE),
				X1(Parachain(1)),
				X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: ALICE.into(),
				}),
				vec![ConcreteFungible { id: Null, amount: 123 }],
				123,
			));
		});

		ParaA::execute_with(|| {
			// free execution, full amount received
			assert_eq!(
				pallet_balances::Pallet::<para::Runtime>::free_balance(&ALICE),
				INITIAL_BALANCE + 123
			);

			print_events::<para::Runtime>("ParaA");
		});
	}

	#[test]
	fn dmp() {
		MockNet::reset();

		let remark = para::Call::System(frame_system::Call::<para::Runtime>::remark_with_event(vec![1, 2, 3]));
		Relay::execute_with(|| {
			assert_ok!(RelayChainPalletXcm::send_xcm(
				Null,
				X1(Parachain(1)),
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: INITIAL_BALANCE as u64,
					call: remark.encode().into(),
				},
			));
		});

		ParaA::execute_with(|| {
			print_events::<para::Runtime>("ParaA");
		});
	}

	#[test]
	fn ump() {
		MockNet::reset();

		let remark = relay::Call::System(frame_system::Call::<relay::Runtime>::remark_with_event(vec![1, 2, 3]));
		ParaA::execute_with(|| {
			assert_ok!(ParachainPalletXcm::send_xcm(
				Null,
				X1(Parent),
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: INITIAL_BALANCE as u64,
					call: remark.encode().into(),
				},
			));
		});

		Relay::execute_with(|| {
			print_events::<relay::Runtime>("RelayChain");
		});
	}

	// // NOTE: XCMP won't work until `https://github.com/paritytech/cumulus/pull/510` fixed.
	// #[test]
	// fn xcmp() {
	// 	MockNet::reset();

	// 	let remark =
	// para::Call::System(frame_system::Call::<para::Runtime>::
	// remark_with_event(vec![1, 2, 3])); 	ParaA::execute_with(|| {
	// 		assert_ok!(ParachainPalletXcm::send_xcm(
	// 			Null,
	// 			X2(Parent, Parachain(2)),
	// 			Transact {
	// 				origin_type: OriginKind::SovereignAccount,
	// 				require_weight_at_most: INITIAL_BALANCE as u64,
	// 				call: remark.encode().into(),
	// 			},
	// 		));
	// 	});

	// 	ParaB::execute_with(|| {
	// 		print_events::<para::Runtime>("ParaB");
	// 	});
	// }
}
