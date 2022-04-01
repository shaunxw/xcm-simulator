use frame_support::traits::GenesisBuild;
use sp_runtime::AccountId32;

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

decl_test_relay_chain! {
	pub struct KusamaNet {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = kusama_ext(),
	}
}

decl_test_parachain! {
	pub struct YayoiPumpkin {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		XcmpMessageHandler = yayoi::XcmpQueue,
		DmpMessageHandler = yayoi::DmpQueue,
		new_ext = yayoi_ext(1),
	}
}

decl_test_parachain! {
	pub struct YayoiMushroom {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		XcmpMessageHandler = yayoi::XcmpQueue,
		DmpMessageHandler = yayoi::DmpQueue,
		new_ext = yayoi_ext(2),
	}
}

decl_test_parachain! {
	pub struct YayoiOctopus {
		Runtime = yayoi::Runtime,
		Origin = yayoi::Origin,
		XcmpMessageHandler = yayoi::XcmpQueue,
		DmpMessageHandler = yayoi::DmpQueue,
		new_ext = yayoi_ext(3),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = KusamaNet,
		parachains = vec![
			(1, YayoiPumpkin),
			(2, YayoiMushroom),
			(3, YayoiOctopus),
		],
	}
}

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

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
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 10u32,
		validation_upgrade_delay: 10,
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
		ump_service_total_weight: 4 * 1_000_000_000,
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
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
	use codec::Encode;

	use cumulus_primitives_core::ParaId;
	use frame_support::{assert_ok, traits::Currency};
	use sp_runtime::traits::AccountIdConversion;
	use xcm::{latest::prelude::*, VersionedMultiLocation, VersionedXcm};
	use xcm_emulator::TestExt;

	#[test]
	fn dmp() {
		Network::reset();

		let remark = yayoi::Call::System(frame_system::Call::<yayoi::Runtime>::remark_with_event {
			remark: "Hello from Kusama!".as_bytes().to_vec(),
		});
		KusamaNet::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::Origin::root(),
				Some(0)
			));
			assert_ok!(kusama_runtime::XcmPallet::send_xcm(
				Here,
				Parachain(1),
				Xcm(vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: INITIAL_BALANCE as u64,
					call: remark.encode().into(),
				}]),
			));
		});

		YayoiPumpkin::execute_with(|| {
			use yayoi::{Event, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
			)));
		});
	}

	#[test]
	fn ump() {
		Network::reset();

		KusamaNet::execute_with(|| {
			let _ = kusama_runtime::Balances::deposit_creating(&ParaId::from(1).into_account(), 1_000_000_000_000);
		});

		let remark = kusama_runtime::Call::System(frame_system::Call::<kusama_runtime::Runtime>::remark_with_event {
			remark: "Hello from Pumpkin!".as_bytes().to_vec(),
		});
		YayoiPumpkin::execute_with(|| {
			assert_ok!(yayoi::PolkadotXcm::send_xcm(
				Here,
				Parent,
				Xcm(vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: INITIAL_BALANCE as u64,
					call: remark.encode().into(),
				}]),
			));
		});

		KusamaNet::execute_with(|| {
			use kusama_runtime::{Event, System};
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
			)));
		});
	}

	#[test]
	fn xcmp() {
		Network::reset();

		let remark = yayoi::Call::System(frame_system::Call::<yayoi::Runtime>::remark_with_event {
			remark: "Hello from Pumpkin!".as_bytes().to_vec(),
		});
		YayoiPumpkin::execute_with(|| {
			assert_ok!(yayoi::PolkadotXcm::send_xcm(
				Here,
				MultiLocation::new(1, X1(Parachain(2))),
				Xcm(vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 10_000_000,
					call: remark.encode().into(),
				}]),
			));
		});

		YayoiMushroom::execute_with(|| {
			use yayoi::{Event, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
			)));
		});
	}

	#[test]
	fn xcmp_through_a_parachain() {
		use yayoi::{Call, PolkadotXcm, Runtime};

		Network::reset();

		// The message goes through: Pumpkin --> Mushroom --> Octopus
		let remark = Call::System(frame_system::Call::<Runtime>::remark_with_event {
			remark: "Hello from Pumpkin!".as_bytes().to_vec(),
		});
		let send_xcm_to_octopus = Call::PolkadotXcm(pallet_xcm::Call::<Runtime>::send {
			dest: Box::new(VersionedMultiLocation::V1(MultiLocation::new(1, X1(Parachain(3))))),
			message: Box::new(VersionedXcm::V2(Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: 10_000_000,
				call: remark.encode().into(),
			}]))),
		});
		YayoiPumpkin::execute_with(|| {
			assert_ok!(PolkadotXcm::send_xcm(
				Here,
				MultiLocation::new(1, X1(Parachain(2))),
				Xcm(vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 100_000_000,
					call: send_xcm_to_octopus.encode().into(),
				}]),
			));
		});

		YayoiMushroom::execute_with(|| {
			use yayoi::{Event, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _)))));
		});

		YayoiOctopus::execute_with(|| {
			use yayoi::{Event, System};
			// execution would fail, but good enough to check if the message is received
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events()
				.iter()
				.any(|r| matches!(r.event, Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail(_, _)))));
		});
	}

	#[test]
	fn deduplicate_dmp() {
		Network::reset();
		KusamaNet::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::Origin::root(),
				Some(0)
			));
		});

		kusama_send_rmrk("Kusama", 2);
		parachain_receive_and_reset_events(true);

		// a different dmp message in same relay-parent-block allow execution.
		kusama_send_rmrk("Polkadot", 1);
		parachain_receive_and_reset_events(true);

		// same dmp message with same relay-parent-block wouldn't execution
		kusama_send_rmrk("Kusama", 1);
		parachain_receive_and_reset_events(false);

		// different relay-parent-block allow dmp message execution
		KusamaNet::execute_with(|| kusama_runtime::System::set_block_number(2));

		kusama_send_rmrk("Kusama", 1);
		parachain_receive_and_reset_events(true);

		// reset can send same dmp message again
		Network::reset();
		KusamaNet::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::Origin::root(),
				Some(0)
			));
		});

		kusama_send_rmrk("Kusama", 1);
		parachain_receive_and_reset_events(true);
	}

	fn kusama_send_rmrk(msg: &str, count: u32) {
		let remark = yayoi::Call::System(frame_system::Call::<yayoi::Runtime>::remark_with_event {
			remark: msg.as_bytes().to_vec(),
		});
		KusamaNet::execute_with(|| {
			for _ in 0..count {
				assert_ok!(kusama_runtime::XcmPallet::send_xcm(
					Here,
					Parachain(1),
					Xcm(vec![Transact {
						origin_type: OriginKind::SovereignAccount,
						require_weight_at_most: INITIAL_BALANCE as u64,
						call: remark.encode().into(),
					}]),
				));
			}
		});
	}

	fn parachain_receive_and_reset_events(received: bool) {
		YayoiPumpkin::execute_with(|| {
			use yayoi::{Event, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			if received {
				assert!(System::events().iter().any(|r| matches!(
					r.event,
					Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
				)));

				System::reset_events();
			} else {
				assert!(System::events().iter().all(|r| !matches!(
					r.event,
					Event::System(frame_system::Event::Remarked { sender: _, hash: _ })
				)));
			}
		});
	}
}
