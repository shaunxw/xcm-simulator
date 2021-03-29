use paste::paste;

use sp_io::TestExternalities;
use sp_std::cell::RefCell;

use cumulus_primitives_core::{
	HrmpMessageHandler, InboundHrmpMessage,
};

pub mod parachain;
pub mod relay_chain;
pub mod traits;

pub use parachain::*;
pub use relay_chain::*;
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
				use runtime_parachains::ump::{UmpSink, XcmSink};

				println!("Ump sent: from {:?}, msg {:?}", from, msg);

				Self::execute_with(|| {
					XcmSink::<relay_default::XcmConfig>::process_upward_message(from.into(), msg);
				});

				Ok(())
			}
		}
	};
}

macro_rules! decl_test_parachain {
	(
		pub struct $name:ident {
			new_ext = $new_ext:expr,
			para_id = $para_id:expr,
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
					para_default::XcmHandler::handle_hrmp_message(
						from.into(),
						InboundHrmpMessage { sent_at: 1, data: msg }
					);
				});
				Ok(())
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
			pub static $ext_name: RefCell<TestExternalities> = RefCell::new($new_ext);
		}

		impl $crate::traits::TestExt for $name {
			fn new_ext() -> sp_io::TestExternalities {
				$new_ext
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				$ext_name.with(|v| v.borrow_mut().execute_with(execute))
			}
		}
	}
}

decl_test_parachain! {
	pub struct MockAcala {
		new_ext = parachain::ext::<para_default::Runtime>(111),
		para_id = 1,
	}
}

#[macro_export]
macro_rules! decl_sim_network {
	(
		pub struct $name:ident {
			relay_chain = default,
			parachains = vec![ $( ($para_id:expr, $parachain:ty), )* ],
		}
	) => {
		pub struct $name;

		decl_test_relay_chain! {
			pub struct MockRelay {
				new_ext = relay_default::ext(),
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

decl_sim_network! {
	pub struct SimNetwork {
		relay_chain = default,
		parachains = vec![(111, MockAcala),],
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	#[test]
// 	fn try_hrmp() {
// 		Sim::reset_ext();

// 		Sim::execute_with(111, || {
// 			let hrmp_result = <para_default::XcmHandler as SendXcm>::send_xcm(
// 				(Junction::Parent, Junction::Parachain { id: 222 }).into(),
// 				Xcm::Transact {
// 					origin_type: OriginKind::Native,
// 					call: vec![1],
// 				},
// 			);
// 			println!("---- sending hrmp: {:?}", hrmp_result);

// 			println!("-------- 111 events");
// 			para_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});

// 		Sim::execute_with(222, || {
// 			println!("-------- 222 events");
// 			para_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});

// 		Sim::reset_ext();

// 		Sim::execute_with(111, || {
// 			println!("-------- 111 events");
// 			para_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});

// 		Sim::execute_with(222, || {
// 			println!("-------- 222 events");
// 			para_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});
// 	}

// 	#[test]
// 	fn try_ump() {
// 		Sim::reset_ext();

// 		Sim::execute_with(222, || {
// 			let sending_result = <para_default::XcmHandler as SendXcm>::send_xcm(
// 				Junction::Parent.into(),
// 				Xcm::Transact {
// 					origin_type: OriginKind::Native,
// 					call: vec![1],
// 				},
// 			);
// 			println!("-------- sending to relay chain: {:?}", sending_result);

// 			para_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});

// 		Sim::relay_chain_execute_with(|| {
// 			relay_default::System::events()
// 				.iter()
// 				.for_each(|r| println!(">>> {:?}", r.event));
// 		});
// 	}
// }