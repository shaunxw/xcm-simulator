#[macro_export]
macro_rules! decl_integration_test_relay_chain {
	(
		pub struct $name:ident {
			Runtime = $runtime:path,
			XcmConfig = $xcm_config:path,
			new_ext = $new_ext:expr,
		}
	) => {
		pub struct $name;

		$crate::__impl_ext_for_relay_chain!($name, $runtime, $new_ext);

		impl $crate::HandleUmpMessage for $name {
			fn handle_ump_message(from: $crate::ParaId, msg: &[u8], max_weight: $crate::Weight) {
				use $crate::ump::UmpSink;
				use $crate::TestExt;

				Self::execute_with(|| {
					let _ = $crate::ump::XcmSink::<$crate::XcmExecutor<$xcm_config>, $runtime>::process_upward_message(
						from, msg, max_weight,
					);
				});
			}
		}
	};
}

#[macro_export]
macro_rules! decl_integration_test_parachain {
	(
		pub struct $name:ident {
			Runtime = $runtime:path,
			Origin = $origin:path,
			new_ext = $new_ext:expr,
		}
	) => {
		pub struct $name;

		$crate::__impl_ext_for_parachain!($name, $runtime, $origin, $new_ext);

		impl $crate::HandleXcmpMessage for $name {
			fn handle_xcmp_message(from: $crate::ParaId, at_relay_block: u32, msg: &[u8], max_weight: $crate::Weight) {
				use $crate::cumulus_primitives_core::XcmpMessageHandler;
				use $crate::TestExt;

				$name::execute_with(|| {
					$crate::cumulus_pallet_xcmp_queue::Pallet::<$runtime>::handle_xcmp_messages(
						vec![(from, at_relay_block, msg)].into_iter(),
						max_weight,
					);
				});
			}
		}

		impl $crate::HandleDmpMessage for $name {
			fn handle_dmp_message(at_relay_block: u32, msg: Vec<u8>, max_weight: $crate::Weight) {
				use $crate::cumulus_primitives_core::DmpMessageHandler;
				use $crate::TestExt;

				$name::execute_with(|| {
					$crate::cumulus_pallet_dmp_queue::Pallet::<$runtime>::handle_dmp_messages(
						vec![(at_relay_block, msg)].into_iter(),
						max_weight,
					);
				});
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
							let downward_messages = <$runtime>::dmq_contents(para_id.into());
							for msg in downward_messages {
								_handle_dmp_message(para_id, msg.sent_at, msg.msg);
							}

							// note: no need to handle horizontal messages, as the simulator directly sends
							// them to dest (not relayed).
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
						use $crate::cumulus_primitives_core::runtime_decl_for_CollectCollationInfo::CollectCollationInfo;
						use $crate::Hooks;

						// get messages
						$crate::cumulus_pallet_parachain_system::Pallet::<$runtime>::on_finalize(1);
						let collation_info = <$runtime>::collect_collation_info();

						// send Messages
						// TODO: xcmp
						let para_id = $crate::parachain_info::Pallet::<$runtime>::get();
						for msg in collation_info.upward_messages {
							_handle_ump_message(para_id, msg);
						}

						// clean messages
						$crate::cumulus_pallet_parachain_system::Pallet::<$runtime>::on_initialize(1);
					})
				});

				r
			}
		}
	};
}

#[macro_export]
macro_rules! decl_integration_test_network {
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

		fn _handle_dmp_message(para_id: u32, sent_at: u32, msg: Vec<u8>) {
			use $crate::HandleDmpMessage;

			match para_id {
				$(
					// TODO: update max weight
					$para_id => <$parachain>::handle_dmp_message(sent_at, msg, $crate::Weight::max_value()),
				)*
				_ => unreachable!(),
			}
		}

		fn _handle_xcmp_message(para_id: u32, from_para_id: $crate::ParaId, sent_at: u32, msg: Vec<u8>) {
			use $crate::HandleXcmpMessage;

			match para_id {
				$(
					// TODO: update max weight
					$para_id => <$parachain>::handle_xcmp_message(
						from_para_id,
						sent_at,
						&msg[..],
						$crate::Weight::max_value(),
					),
				)*
				_ => unreachable!(),
			}
		}

		fn _handle_ump_message(para_id: $crate::ParaId, msg: Vec<u8>) {
			use $crate::HandleUmpMessage;

			<$relay_chain>::handle_ump_message(para_id, &msg[..], $crate::Weight::max_value());
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
			for recipient_para_id in [ $( $para_id, )* ] {
				let recipient_para_id = $crate::ParaId::from(recipient_para_id);
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
