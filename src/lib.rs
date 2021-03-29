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

//TODO: pattern
thread_local! {
	pub static EXT_RELAY: RefCell<TestExternalities> = RefCell::new(relay_default::ext());
	pub static EXT_111: RefCell<TestExternalities> = RefCell::new(parachain_ext::<para_default::Runtime>(111));
	pub static EXT_222: RefCell<TestExternalities> = RefCell::new(parachain_ext::<para_default::Runtime>(222));
}

pub struct Sim;
impl Sim {
	pub fn reset_ext() {
		//TODO: pattern
		EXT_111.with(|v| *v.borrow_mut() = parachain_ext::<para_default::Runtime>(111));
		EXT_222.with(|v| *v.borrow_mut() = parachain_ext::<para_default::Runtime>(222));
		EXT_RELAY.with(|v| *v.borrow_mut() = relay_default::ext());
	}

	pub fn execute_with<R>(para_id: u32, execute: impl FnOnce() -> R) -> R {
		match para_id {
			//TODO: pattern
			111 => EXT_111.with(|v| v.borrow_mut().execute_with(execute)),
			222 => EXT_222.with(|v| v.borrow_mut().execute_with(execute)),
			_ => unreachable!("ext has been set"),
		}
	}

	pub fn relay_chain_execute_with<R>(execute: impl FnOnce() -> R) -> R {
		EXT_RELAY.with(|v| v.borrow_mut().execute_with(execute))
	}

	fn send_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()> {
		use runtime_parachains::ump::{UmpSink, XcmSink};

		println!("Sim ump sent: from {:?}, msg {:?}", from, msg);

		Self::relay_chain_execute_with(|| {
			XcmSink::<relay_default::XcmConfig>::process_upward_message(from.into(), msg);
		});

		Ok(())
	}

	fn send_hrmp_msg(from: u32, to: u32, msg: Vec<u8>) -> Result<(), ()> {
		println!("Sim hrmp sent: from {:?}, to {:?}, msg {:?}", from, to, msg);
		match to {
			//TODO: pattern
			111 | 222 => {
				Self::execute_with(to, || {
					para_default::XcmHandler::handle_hrmp_message(from.into(), InboundHrmpMessage { sent_at: 10, data: msg });
				});
				Ok(())
			}
			_ => Err(()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn try_hrmp() {
		Sim::reset_ext();

		Sim::execute_with(111, || {
			let hrmp_result = <para_default::XcmHandler as SendXcm>::send_xcm(
				(Junction::Parent, Junction::Parachain { id: 222 }).into(),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: vec![1],
				},
			);
			println!("---- sending hrmp: {:?}", hrmp_result);

			println!("-------- 111 events");
			para_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});

		Sim::execute_with(222, || {
			println!("-------- 222 events");
			para_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});

		Sim::reset_ext();

		Sim::execute_with(111, || {
			println!("-------- 111 events");
			para_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});

		Sim::execute_with(222, || {
			println!("-------- 222 events");
			para_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});
	}

	#[test]
	fn try_ump() {
		Sim::reset_ext();

		Sim::execute_with(222, || {
			let sending_result = <para_default::XcmHandler as SendXcm>::send_xcm(
				Junction::Parent.into(),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: vec![1],
				},
			);
			println!("-------- sending to relay chain: {:?}", sending_result);

			para_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});

		Sim::relay_chain_execute_with(|| {
			relay_default::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});
	}
}

#[macro_export]
macro_rules! decl_test_chain {
	// entry point
	(
		pub struct $name:ident {
			new_ext = $new_ext:expr,
			$( para_id = $para_id:expr, )?
		}
	) => {
		paste! {
			$crate::decl_test_chain!(@impl
				pub struct $name {
					new_ext = $new_ext,
					ext_name = [<EXT_ $name:upper>],
					$( para_id = $para_id, )?
				}
			);
		}
	};

	// branch - impl relay chain
	(@impl
		pub struct $name:ident {
			new_ext = $new_ext:expr,
			ext_name = $ext_name:ident,
		}
	) => {
		$crate::decl_test_chain!(@define_ext $ext_name; $new_ext;);

		$crate::decl_test_chain!(@define_struct $name;);
		$crate::decl_test_chain!(@impl_test_ext $name; $ext_name; $new_ext; );
		$crate::decl_test_chain!(@impl_ump_msg_handler $name;);
	};

	// branch - impl parachain
	(@impl
		pub struct $name:ident {
			new_ext = $new_ext:expr,
			ext_name = $ext_name:ident,
			para_id = $para_id:expr,
		}
	) => {
		$crate::decl_test_chain!(@define_ext $ext_name; $new_ext;);

		$crate::decl_test_chain!(@define_struct $name;);
		$crate::decl_test_chain!(@impl_test_ext $name; $ext_name; $new_ext; );
		$crate::decl_test_chain!(@impl_get_para_id $name; $para_id;);
		$crate::decl_test_chain!(@impl_hrmp_msg_handler $name;);
	};

	(@define_ext
		$ext_name:ident;
		$new_ext:expr;
	) => {
		thread_local! {
			pub static $ext_name: RefCell<TestExternalities> = RefCell::new($new_ext);
		}
	};

	(@define_struct
		$name:ident;
	) => {
		pub struct $name;
	};

	//TODO: use non-default relay chain runtime
	(@impl_ump_msg_handler
		$name:ident;
	) => {
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

	(@impl_get_para_id
		$name:ident;
		$para_id:expr;
	) => {
		impl $crate::traits::GetParaId for $name {
			fn para_id() -> u32 {
				$para_id
			}
		}
	};

	//TODO: use non-default parachain runtime
	(@impl_hrmp_msg_handler
		$name:ident;
	) => {
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
	};

	(@impl_test_ext
		$name:ident;
		$ext_name:ident;
		$new_ext:expr;
	) => {
		impl $crate::traits::TestExt for $name {
			fn new_ext() -> sp_io::TestExternalities {
				$new_ext
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				$ext_name.with(|v| v.borrow_mut().execute_with(execute))
			}
		}
	};
}

decl_test_chain! {
	pub struct MockAcala {
		new_ext = parachain_ext::<para_default::Runtime>(111),
		para_id = 1,
	}
}

decl_test_chain! {
	pub struct MockRelay {
		new_ext = relay_default::ext(),
	}
}
