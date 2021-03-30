use paste::paste;

use sp_io::TestExternalities;
use sp_std::cell::RefCell;

use cumulus_primitives_core::{HrmpMessageHandler, InboundHrmpMessage};

pub mod parachain;
pub mod relay_chain;
pub mod traits;

pub use traits::*;

macro_rules! decl_test_relay_chain {
	(
		pub struct $name:ident {
			new_ext = $new_ext:expr,
		}
	) => {
		pub struct $name;

		$crate::impl_ext!($name, $new_ext);

		impl $crate::traits::UmpMsgHandler for $name {
			fn handle_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
				use runtime_parachains::ump::UmpSink;
				use $crate::relay_chain::default;

				println!("Ump sent: from {:?}, msg {:?}", from, msg);

				Self::execute_with(|| {
					default::UmpSink::process_upward_message(from.into(), msg);
				});

				Ok(())
			}
		}
	};
}

#[macro_export]
macro_rules! decl_test_parachain {
	(
		pub struct $name:ident {
			new_ext = $new_ext:expr,
			para_id = $para_id:expr,
		}
		pub mod $para_mod:ident {
			test_network = $test_network:path,
			xcm_config = { $( $xcm_config:tt )* },
			extra_config = { $( $extra_config:tt )* },
			extra_modules = { $( $extra_modules:tt )* },
		}
	) => {
		pub struct $name;

		$crate::impl_ext!($name, $new_ext);

		impl $crate::traits::GetParaId for $name {
			fn para_id() -> u32 {
				$para_id
			}
		}

		impl HrmpMsgHandler for $name {
			fn handle_hrmp_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
				$name::execute_with(|| {
					//TODO: `sent_at` - check with runtime
					$para_mod::XcmHandler::handle_hrmp_message(
						from.into(),
						InboundHrmpMessage { sent_at: 1, data: msg }
					);
				});
				Ok(())
			}
		}

		$crate::construct_parachain_runtime! {
			pub mod $para_mod {
				test_network = $test_network,
				xcm_config = { $( $xcm_config )* },
				extra_config = { $( $extra_config )* },
				extra_modules = { $( $extra_modules )* },
			}
		}
	}
}

#[macro_export]
macro_rules! impl_ext {
	// entry point: generate ext name
	($name:ident, $new_ext:expr) => {
		paste! {
			$crate::impl_ext!(@impl $name, $new_ext, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $new_ext:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: RefCell<$crate::TestExternalities> = RefCell::new($new_ext);
		}

		impl $crate::traits::TestExt for $name {
			fn new_ext() -> $crate::TestExternalities {
				$new_ext
			}

			fn reset_ext() {
				$ext_name.with(|v| *v.borrow_mut() = $new_ext);
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				$ext_name.with(|v| v.borrow_mut().execute_with(execute))
			}
		}
	};
}

#[macro_export]
macro_rules! decl_test_network {
	(
		pub struct $name:ident {
			relay_chain = default,
			parachains = vec![ $( ($para_id:expr, $parachain:ty), )* ],
		}
	) => {
		pub struct $name;

		impl $name {
			fn reset() {
				MockRelay::reset_ext();
				$( <$parachain>::reset_ext(); )*
			}
		}

		decl_test_relay_chain! {
			pub struct MockRelay {
				new_ext = $crate::relay_chain::default::ext(),
			}
		}

		impl $crate::traits::XcmRelay for $name {
			fn send_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
				MockRelay::handle_ump_msg(from, msg)
			}

			fn send_hrmp_msg(from: u32, to: u32, msg: Vec<u8>) -> Result<(), ()> {
				match to {
					$( $para_id => <$parachain>::handle_hrmp_msg(from, msg), )*
					_ => Err(()),
				}
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;

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
