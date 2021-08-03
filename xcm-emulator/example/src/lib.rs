use codec::Encode;

use cumulus_primitives_core::ParaId;
use frame_support::traits::{Currency, GenesisBuild};
use sp_runtime::{traits::AccountIdConversion, AccountId32};

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
		Junction::{Parachain, Parent},
		MultiAsset::*,
		MultiLocation::*,
		Order::BuyExecution,
		OriginKind,
		Xcm::*,
	};
	use xcm_emulator::TestExt;

	#[test]
	fn dmp() {
		Network::reset();

		let remark = yayoi::Call::System(frame_system::Call::<yayoi::Runtime>::remark_with_event(
			"Hello from Kusama!".as_bytes().to_vec(),
		));
		Kusama::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::send_xcm(
				Null,
				Parachain(1).into(),
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 10_000_000,
					call: remark.encode().into(),
				}
			));
		});

		YayoiPumpkin::execute_with(|| {
			use yayoi::{Event, System};
			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
		});
	}

	#[test]
	fn ump() {
		Network::reset();

		Kusama::execute_with(|| {
			let _ = kusama_runtime::Balances::deposit_creating(&ParaId::from(1).into_account(), 1_000_000_000_000);
		});

		let remark = kusama_runtime::Call::System(frame_system::Call::<kusama_runtime::Runtime>::remark_with_event(
			"Hello from Kusama!".as_bytes().to_vec(),
		));
		YayoiPumpkin::execute_with(|| {
			assert_ok!(yayoi::PolkadotXcm::send_xcm(
				Null,
				Parent.into(),
				WithdrawAsset {
					assets: vec![ConcreteFungible {
						id: Null,
						amount: 1_000_000_000_000
					}],
					effects: vec![BuyExecution {
						fees: All,
						weight: 10_000_000,
						debt: 10_000_000,
						halt_on_error: true,
						xcm: vec![Transact {
							origin_type: OriginKind::SovereignAccount,
							require_weight_at_most: 1_000_000_000,
							call: remark.encode().into(),
						}],
					}]
				}
			));
		});

		Kusama::execute_with(|| {
			use kusama_runtime::{Event, System};
			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
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
					require_weight_at_most: 10_000_000,
					call: remark.encode().into(),
				},
			));
		});

		YayoiMushroom::execute_with(|| {
			use yayoi::{Event, System};
			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
		});
	}

	#[test]
	fn xcmp_through_a_parachain() {
		use yayoi::{Call, PolkadotXcm, Runtime};

		Network::reset();

		// The message goes through: Pumpkin --> Mushroom --> Octopus
		let remark = Call::System(frame_system::Call::<Runtime>::remark_with_event(
			"Hello from Pumpkin!".as_bytes().to_vec(),
		));
		let send_xcm_to_octopus = Call::PolkadotXcm(pallet_xcm::Call::<Runtime>::send(
			X2(Parent, Parachain(3)),
			Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: 10_000_000,
				call: remark.encode().into(),
			},
		));
		YayoiPumpkin::execute_with(|| {
			assert_ok!(PolkadotXcm::send_xcm(
				Null,
				X2(Parent, Parachain(2)),
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 100_000_000,
					call: send_xcm_to_octopus.encode().into(),
				},
			));
		});

		YayoiMushroom::execute_with(|| {
			use yayoi::{Event, System};
			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _)))));
		});

		YayoiOctopus::execute_with(|| {
			use yayoi::{Event, System};
			// execution would fail, but good enough to check if the message is received
			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail(_, _)))));
		});
	}
}
