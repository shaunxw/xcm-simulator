use codec::Encode;

use frame_support::traits::GenesisBuild;
use sp_runtime::AccountId32;

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

decl_test_parachain! {
	pub struct Statemine {
		Runtime = statemine_runtime::Runtime,
		Origin = statemine_runtime::Origin,
		new_ext = statemine_ext(),
	}
}

decl_test_relay_chain! {
	pub struct Kusama {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::XcmConfig,
		new_ext = kusama_ext(),
	}
}

decl_test_parachain! {
	pub struct YayoiPumpkin {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		new_ext = yayoi_ext(1),
	}
}

decl_test_parachain! {
	pub struct YayoiMushroom {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		new_ext = yayoi_ext(2),
	}
}

decl_test_parachain! {
	pub struct YayoiOctopus {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		new_ext = yayoi_ext(3),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = Kusama,
		parachains = vec![
			(1, YayoiPumpkin),
			(2, YayoiMushroom),
			(3, YayoiOctopus),
			(1000, Statemine),
		],
	}
}

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub fn statemine_ext() -> sp_io::TestExternalities {
	use statemine_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let parachain_info_config = parachain_info::GenesisConfig {
		parachain_id: 1_000.into(),
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

pub fn yayoi_ext(para_id: u32) -> sp_io::TestExternalities {
	use yayoi::{Runtime, System};

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

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<polkadot_primitives::v1::BlockNumber> {
	use polkadot_primitives::v1::{MAX_CODE_SIZE, MAX_POV_SIZE};

	polkadot_runtime_parachains::configuration::HostConfiguration {
		validation_upgrade_frequency: 1u32,
		validation_upgrade_delay: 1,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		// this is approximatelly 4ms.
		//
		// Same as `4 * frame_support::weights::WEIGHT_PER_MILLIS`. We don't bother with
		// an import since that's a made up number and should be replaced with a constant
		// obtained by benchmarking anyway.
		ump_service_total_weight: 4 * 1_000_000_000,
		max_upward_message_size: 1024 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_open_request_ttl: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		..Default::default()
	}
}

pub fn kusama_ext() -> sp_io::TestExternalities {
	use kusama_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::assert_ok;
	use xcm::v0::{
		Junction::{self, Parachain, Parent},
		MultiAsset::*,
		MultiLocation::*,
		NetworkId, OriginKind,
		Xcm::*,
	};
	use xcm_emulator::TestExt;

	fn print_events<T: frame_system::Config>(context: &str) {
		println!("------ {:?} events ------", context);
		frame_system::Pallet::<T>::events().iter().for_each(|r| {
			println!("{:?}", r.event);
		});
	}

	#[test]
	fn dmp() {
		Network::reset();

		Kusama::execute_with(|| {
			use kusama_runtime::{Origin, Runtime, XcmPallet};

			assert_ok!(XcmPallet::teleport_assets(
				Origin::signed(ALICE),
				Parachain(1000).into(),
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: ALICE.into(),
				}
				.into(),
				vec![ConcreteFungible {
					id: Null,
					amount: 1_000_000_000_000
				}],
				1_000_000_000
			));
			print_events::<Runtime>("Kusama");
		});

		Statemine::execute_with(|| {
			print_events::<statemine_runtime::Runtime>("Statemine");
		});
	}

	#[test]
	fn ump() {
		Network::reset();

		Kusama::execute_with(|| {
			use kusama_runtime::{Balances, CheckAccount, Origin};

			assert_ok!(Balances::set_balance(
				Origin::root(),
				CheckAccount::get().into(),
				1_000_000_000_000,
				0
			));
		});

		Statemine::execute_with(|| {
			use statemine_runtime::{Origin, PolkadotXcm, Runtime};

			assert_ok!(PolkadotXcm::teleport_assets(
				Origin::signed(ALICE),
				Parent.into(),
				Junction::AccountId32 {
					network: NetworkId::Kusama,
					id: ALICE.into(),
				}
				.into(),
				vec![ConcreteFungible {
					id: Parent.into(),
					amount: 1_000_000_000_000,
				}],
				3_000_000_000
			));
			print_events::<Runtime>("Statemine");
		});

		Kusama::execute_with(|| {
			print_events::<kusama_runtime::Runtime>("Kusama");
		});
	}

	#[test]
	fn xcmp() {
		Network::reset();

		let remark = yayoi::Call::System(frame_system::Call::<yayoi::Runtime>::remark_with_event(
			"Hello from Pumpkin!".as_bytes().to_vec(),
		));
		YayoiPumpkin::execute_with(|| {
			assert_ok!(yayoi::PolkadotXcm::send_xcm(
				Null,
				X2(Parent, Parachain(2)),
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 2_000_000_000,
					call: remark.encode().into(),
				},
			));
			print_events::<yayoi::Runtime>("Yayoi Pumpkin");
		});

		YayoiMushroom::execute_with(|| {
			print_events::<yayoi::Runtime>("Yayoi Mushroom");
		});
	}
}
