pub use paste;
pub use codec;

pub use frame_support;
pub use frame_system;
pub use pallet_balances;
pub use sp_core;
pub use sp_io;
pub use sp_runtime;
pub use sp_std;

pub use cumulus_pallet_xcm_handler;
pub use cumulus_primitives_core;
pub use parachain_info;
pub use polkadot_parachain;
pub use runtime_parachains;
pub use xcm;
pub use xcm_builder;
pub use xcm_executor;

pub mod parachain;
pub mod relay_chain;
pub mod traits;

pub use traits::*;

pub mod prelude {
	pub use crate::parachain;
	pub use crate::relay_chain;
	pub use crate::traits::*;
}

#[macro_export]
macro_rules! __decl_test_relay_chain {
	(
		pub struct $name:ident {
			new_ext = $new_ext:expr,
		}
		pub mod $relay_mod:ident {
			test_network = $test_network:path,
		}
	) => {
		pub struct $name;

		$crate::__impl_ext!($name, $new_ext);

		impl $crate::traits::UmpMsgHandler for $name {
			fn handle_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
				use $crate::runtime_parachains::ump::UmpSink;

				Self::execute_with(|| {
					$relay_mod::UmpSink::process_upward_message(from.into(), msg);
				});

				Ok(())
			}
		}

		$crate::__construct_relay_chain_runtime! {
			pub mod $relay_mod {
				test_network = $test_network,
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

		$crate::__impl_ext!($name, $new_ext);

		impl $crate::traits::GetParaId for $name {
			fn para_id() -> u32 {
				$para_id
			}
		}

		impl $crate::HrmpMsgHandler for $name {
			fn handle_hrmp_msg(from: u32, msg: $crate::xcm::VersionedXcm) -> Result<(), ()> {
				use $crate::cumulus_primitives_core::{XcmpMessageHandler, InboundHrmpMessage};

				$name::execute_with(|| {
					//TODO: `sent_at` - check with runtime
					$para_mod::XcmHandler::handle_xcm_message(from.into(), 1, msg);
				});
				Ok(())
			}
		}

		impl $crate::DmpMsgHandler for $name {
			fn handle_dmp_msg(msg: $crate::xcm::v0::Xcm) -> $crate::xcm::v0::Result {
				use $crate::cumulus_primitives_core::{DownwardMessageHandler, InboundDownwardMessage};
				use $crate::codec::Encode;

				$name::execute_with(|| {
					//TODO: `sent_at` - check with runtime
					let dmp_msg = InboundDownwardMessage {
						sent_at: 1,
						msg: $crate::xcm::VersionedXcm::from(msg).encode(),
					};
					$para_mod::XcmHandler::handle_downward_message(dmp_msg);
				});
				Ok(())
			}
		}

		$crate::__construct_parachain_runtime! {
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
macro_rules! __impl_ext {
	// entry point: generate ext name
	($name:ident, $new_ext:expr) => {
		$crate::paste::paste! {
			$crate::__impl_ext!(@impl $name, $new_ext, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $new_ext:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: $crate::sp_std::cell::RefCell<$crate::sp_io::TestExternalities>
				= $crate::sp_std::cell::RefCell::new($new_ext);
		}

		impl $crate::traits::TestExt for $name {
			fn new_ext() -> $crate::sp_io::TestExternalities {
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
			pub fn reset() {
				MockRelay::reset_ext();
				$( <$parachain>::reset_ext(); )*
			}
		}

		$crate::__decl_test_relay_chain! {
			pub struct MockRelay {
				new_ext = $crate::relay_chain::default_ext::<relay::Runtime>(),
			}
			pub mod relay {
				test_network = $name,
			}
		}

		impl $crate::traits::XcmRelay for $name {
			fn send_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
				println!("ump: from {:?}, msg {:?}", from, msg);

				MockRelay::handle_ump_msg(from, msg)
			}

			fn send_hrmp_msg(from: u32, to: u32, msg: $crate::xcm::VersionedXcm) -> Result<(), ()> {
				println!("hrmp: from {:?}, to {:?}, msg {:?}", from, to, msg);

				match to {
					$( $para_id => <$parachain>::handle_hrmp_msg(from, msg), )*
					_ => Err(()),
				}
			}

			fn send_dmp_msg(to: u32, msg: $crate::xcm::v0::Xcm) -> $crate::xcm::v0::Result {
				println!("dmp: to {:?}, msg {:?}", to, msg);

				match to {
					$( $para_id => <$parachain>::handle_dmp_msg(msg), )*
					_ => Err($crate::xcm::v0::Error::CannotReachDestination),
				}
			}
		}
	};
}
