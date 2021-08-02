pub use codec::Encode;
pub use paste;

pub use frame_support::{
	traits::{Get, Hooks},
	weights::Weight,
};
pub use frame_system;
pub use sp_io::TestExternalities;
pub use sp_std::{cell::RefCell, marker::PhantomData};

pub use cumulus_pallet_dmp_queue;
pub use cumulus_pallet_parachain_system;
pub use cumulus_pallet_xcmp_queue;
pub use cumulus_primitives_core::{
	self, relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler, ParaId, PersistedValidationData,
	XcmpMessageHandler,
};
pub use cumulus_primitives_parachain_inherent::ParachainInherentData;
pub use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
pub use parachain_info;

pub use polkadot_primitives;
pub use polkadot_runtime_parachains::{
	dmp,
	ump::{MessageId, UmpSink, XcmSink},
};
pub use xcm::{v0::prelude::*, VersionedXcm};
pub use xcm_executor::XcmExecutor;

pub trait TestExt {
	fn new_ext() -> sp_io::TestExternalities;
	fn reset_ext();
	fn execute_with<R>(execute: impl FnOnce() -> R) -> R;
}

#[macro_export]
macro_rules! decl_test_relay_chain {
	(
		pub struct $name:ident {
			Runtime = $runtime:path,
			XcmConfig = $xcm_config:path,
			new_ext = $new_ext:expr,
		}
	) => {
		pub struct $name;

		$crate::__impl_ext_for_relay_chain!($name, $runtime, $new_ext);

		impl $crate::UmpSink for $name {
			fn process_upward_message(
				origin: $crate::ParaId,
				msg: &[u8],
				max_weight: $crate::Weight,
			) -> Result<$crate::Weight, ($crate::MessageId, $crate::Weight)> {
				use $crate::{TestExt, UmpSink};

				Self::execute_with(|| {
					$crate::XcmSink::<$crate::XcmExecutor<$xcm_config>, $runtime>::process_upward_message(
						origin, msg, max_weight,
					)
				})
			}
		}
	};
}

#[macro_export]
macro_rules! decl_test_parachain {
	(
		pub struct $name:ident {
			Runtime = $runtime:path,
			Origin = $origin:path,
			new_ext = $new_ext:expr,
		}
	) => {
		pub struct $name;

		$crate::__impl_ext_for_parachain!($name, $runtime, $origin, $new_ext);

		impl $crate::XcmpMessageHandler for $name {
			fn handle_xcmp_messages<'a, I: Iterator<Item = ($crate::ParaId, $crate::RelayBlockNumber, &'a [u8])>>(
				iter: I,
				max_weight: $crate::Weight,
			) -> $crate::Weight {
				use $crate::{TestExt, XcmpMessageHandler};

				$name::execute_with(|| {
					$crate::cumulus_pallet_xcmp_queue::Pallet::<$runtime>::handle_xcmp_messages(iter, max_weight)
				})
			}
		}

		impl $crate::DmpMessageHandler for $name {
			fn handle_dmp_messages(
				iter: impl Iterator<Item = ($crate::RelayBlockNumber, Vec<u8>)>,
				max_weight: $crate::Weight,
			) -> $crate::Weight {
				use $crate::{DmpMessageHandler, TestExt};

				$name::execute_with(|| {
					$crate::cumulus_pallet_dmp_queue::Pallet::<$runtime>::handle_dmp_messages(iter, max_weight)
				})
			}
		}
	};
}

#[macro_export]
macro_rules! __impl_ext_for_relay_chain {
	// entry point: generate ext name
	($name:ident, $runtime:path, $new_ext:expr) => {
		$crate::paste::paste! {
			$crate::__impl_ext_for_relay_chain!(@impl $name, $runtime, $new_ext, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $runtime:path, $new_ext:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: $crate::RefCell<$crate::TestExternalities>
				= $crate::RefCell::new($new_ext);
		}

		impl $crate::TestExt for $name {
			fn new_ext() -> $crate::TestExternalities {
				$new_ext
			}

			fn reset_ext() {
				$ext_name.with(|v| *v.borrow_mut() = $new_ext);
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				let r = $ext_name.with(|v| v.borrow_mut().execute_with(execute));

				// send messages if needed
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						use $crate::polkadot_primitives::v1::runtime_decl_for_ParachainHost::ParachainHost;

						//TODO: mark sent count & filter out sent msg
						for para_id in _para_ids() {
							// downward messages
							let downward_messages = <$runtime>::dmq_contents(para_id.into())
								.into_iter()
								.map(|inbound| (inbound.sent_at, inbound.msg));
							if downward_messages.len() == 0 {
								continue;
							}
							_Messenger::send_downward_messages(para_id, downward_messages.into_iter());

							// Note: no need to handle horizontal messages, as the
							// simulator directly sends them to dest (not relayed).
						}
					})
				});

				r
			}
		}
	};
}

#[macro_export]
macro_rules! __impl_ext_for_parachain {
	// entry point: generate ext name
	($name:ident, $runtime:path, $origin:path, $new_ext:expr) => {
		$crate::paste::paste! {
			$crate::__impl_ext_for_parachain!(@impl $name, $runtime, $origin, $new_ext, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $runtime:path, $origin:path, $new_ext:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: $crate::RefCell<$crate::TestExternalities>
				= $crate::RefCell::new($new_ext);
		}

		impl $crate::TestExt for $name {
			fn new_ext() -> $crate::TestExternalities {
				$new_ext
			}

			fn reset_ext() {
				$ext_name.with(|v| *v.borrow_mut() = $new_ext);
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				use $crate::Get;

				// prepare parachain system for messaging
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						let block_number = $crate::frame_system::Pallet::<$runtime>::block_number();
						let para_id = $crate::parachain_info::Pallet::<$runtime>::get();
						let _ = $crate::cumulus_pallet_parachain_system::Pallet::<$runtime>::set_validation_data(
							<$origin>::none(),
							_hrmp_channel_parachain_inherent_data(para_id.into(), 1),
						);
					})
				});

				let r = $ext_name.with(|v| v.borrow_mut().execute_with(execute));

				// send messages if needed
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						use $crate::Hooks;
						type ParachainSystem = $crate::cumulus_pallet_parachain_system::Pallet<$runtime>;

						// get messages
						ParachainSystem::on_finalize(1);
						let collation_info = ParachainSystem::collect_collation_info();

						// send upward messages
						let para_id = $crate::parachain_info::Pallet::<$runtime>::get();
						for msg in collation_info.upward_messages {
							_Messenger::send_upward_message(para_id.into(), &msg[..]);
						}

						// TODO: send horizontal messages

						// clean messages
						ParachainSystem::on_initialize(1);
					})
				});

				r
			}
		}
	};
}

#[macro_export]
macro_rules! decl_test_network {
	(
		pub struct $name:ident {
			relay_chain = $relay_chain:ty,
			parachains = vec![ $( ($para_id:expr, $parachain:ty), )* ],
		}
	) => {
		pub struct $name;

		impl $name {
			pub fn reset() {
				use $crate::TestExt;

				<$relay_chain>::reset_ext();
				$( <$parachain>::reset_ext(); )*
			}
		}

		fn _para_ids() -> Vec<u32> {
			vec![$( $para_id, )*]
		}

		pub struct _Messenger;
		impl _Messenger {
			fn send_downward_messages(to_para_id: u32, iter: impl Iterator<Item = ($crate::RelayBlockNumber, Vec<u8>)>) {
				 use $crate::DmpMessageHandler;

				 match to_para_id {
					$(
						$para_id => { <$parachain>::handle_dmp_messages(iter, $crate::Weight::max_value()); },
					)*
					_ => unreachable!(),
				}
			}

			fn send_horizontal_messages<
				'a,
				I: Iterator<Item = ($crate::ParaId, $crate::RelayBlockNumber, &'a [u8])>,
			>(from_para_id: u32, to_para_id: u32, iter: I) {
				use $crate::XcmpMessageHandler;

				match to_para_id {
					$(
						$para_id => { <$parachain>::handle_xcmp_messages(iter, $crate::Weight::max_value()); },
					)*
					_ => unreachable!(),
				}
			}

			fn send_upward_message(from_para_id: u32, msg: &[u8]) {
				use $crate::UmpSink;
				let _ =  <$relay_chain>::process_upward_message(from_para_id.into(), msg, $crate::Weight::max_value());
			}
		}

		fn _hrmp_channel_parachain_inherent_data(
			para_id: u32,
			relay_parent_number: u32,
		) -> $crate::ParachainInherentData {
			use $crate::cumulus_primitives_core::{relay_chain::v1::HrmpChannelId, AbridgedHrmpChannel};

			let mut sproof = $crate::RelayStateSproofBuilder::default();
			sproof.para_id = para_id.into();

			// egress channel
			let e_index = sproof.hrmp_egress_channel_index.get_or_insert_with(Vec::new);
			for recipient_para_id in &[ $( $para_id, )* ] {
				let recipient_para_id = $crate::ParaId::from(*recipient_para_id);
				if let Err(idx) = e_index.binary_search(&recipient_para_id) {
					e_index.insert(idx, recipient_para_id);
				}

				sproof
					.hrmp_channels
					.entry(HrmpChannelId {
						sender: sproof.para_id,
						recipient: recipient_para_id,
					})
					.or_insert_with(|| AbridgedHrmpChannel {
						max_capacity: 1024,
						max_total_size: 1024 * 1024,
						max_message_size: 1024 * 1024,
						msg_count: 0,
						total_size: 0,
						mqc_head: Option::None,
					});
			}

			let (relay_storage_root, proof) = sproof.into_state_root_and_proof();

			$crate::ParachainInherentData {
				validation_data: $crate::PersistedValidationData {
					parent_head: Default::default(),
					relay_parent_number,
					relay_parent_storage_root: relay_storage_root,
					max_pov_size: Default::default(),
				},
				relay_chain_state: proof,
				downward_messages: Default::default(),
				horizontal_messages: Default::default(),
			}
		}
	};
}
