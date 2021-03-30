fn main() {}

use xcm_simulator::{prelude::*, decl_test_parachain, decl_test_network};
use xcm::v0::{Junction, OriginKind, SendXcm, Xcm};

decl_test_parachain! {
	pub struct MockAcala {
		new_ext = parachain::default_ext::<mock_acala::Runtime>(1),
		para_id = 1,
	}
	pub mod mock_acala {
		test_network = super::TestNetwork,
		xcm_config = { default },
		extra_config = {
			impl orml_nft::Config for Runtime {
				type ClassId = u64;
				type TokenId = u64;
				type ClassData = ();
				type TokenData = ();
			}
		},
		extra_modules = {
			NFT: orml_nft::{Pallet, Storage, Config<T>},
		},
	}
}

decl_test_parachain! {
	pub struct MockLaminar {
		new_ext = parachain::default_ext::<mock_laminar::Runtime>(2),
		para_id = 2,
	}
	pub mod mock_laminar {
		test_network = super::TestNetwork,
		xcm_config = { default },
		extra_config = {},
		extra_modules = {},
	}
}

decl_test_network! {
	pub struct TestNetwork {
		relay_chain = default,
		parachains = vec![
			(1, MockAcala),
			(2, MockLaminar),
		],
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn try_hrmp() {
		TestNetwork::reset();
		MockAcala::execute_with(|| {
			let _ = <mock_acala::XcmHandler as SendXcm>::send_xcm(
				(Junction::Parent, Junction::Parachain { id: 2 }).into(),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: vec![1],
				},
			);
			println!(">>> Acala events:");
			mock_acala::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});
		MockLaminar::execute_with(|| {
			println!(">>> Laminar events:");
			mock_laminar::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});

		TestNetwork::reset();
		println!("------ network reset ------");
		MockAcala::execute_with(|| {
			println!(">>> Acala events:");
			mock_acala::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});

		MockLaminar::execute_with(|| {
			println!(">>> Laminar events:");
			mock_laminar::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});
	}

	#[test]
	fn try_ump() {
		TestNetwork::reset();
		MockAcala::execute_with(|| {
			let _ = <mock_acala::XcmHandler as SendXcm>::send_xcm(
				Junction::Parent.into(),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: vec![1],
				},
			);
			println!(">>> Acala events:");
			mock_acala::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});
		// note: sadly there is no event for ump execution in relay chain https://github.com/paritytech/polkadot/issues/2720
		MockRelay::execute_with(|| {
			println!(">>> Relay chain events:");
			relay_chain::default::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});

		TestNetwork::reset();
		println!("------ network reset ------");
		MockAcala::execute_with(|| {
			let _ = <mock_acala::XcmHandler as SendXcm>::send_xcm(
				Junction::Parent.into(),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: vec![1],
				},
			);
			println!(">>> Acala events:");
			mock_acala::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});
		MockRelay::execute_with(|| {
			println!(">>> relay chain events:");
			relay_chain::default::System::events().iter().for_each(|r| {
				println!("{:?}", r.event);
			});
		});
	}
}
