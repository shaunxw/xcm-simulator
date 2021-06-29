#![allow(clippy::result_unit_err)]

use cumulus_primitives_core::ParaId;
use frame_support::weights::Weight;

pub trait TestExt {
	fn new_ext() -> sp_io::TestExternalities;
	fn reset_ext();
	fn execute_with<R>(execute: impl FnOnce() -> R) -> R;
}

pub trait HandleUmpMessage {
	fn handle_ump_message(from: ParaId, msg: &[u8], max_weight: Weight);
}

pub trait HandleDmpMessage {
	fn handle_dmp_message(at_relay_block: u32, msg: Vec<u8>, max_weight: Weight);
}

pub trait HandleXcmpMessage {
	fn handle_xcmp_message(from: ParaId, at_relay_block: u32, msg: &[u8], max_weight: Weight);
}
