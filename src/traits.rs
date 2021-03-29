pub trait TestExt {
	fn new_ext() -> sp_io::TestExternalities;
	fn execute_with<R>(execute: impl FnOnce() -> R) -> R;
}

pub trait UmpMsgHandler {
	fn handle_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()>;
}

pub trait HrmpMsgHandler {
	fn handle_hrmp_msg(from: u32, msg: Vec<u8>) -> Result<(), ()>;
}

pub trait XcmRelay {
    fn send_ump_msg(from: u32, msg: Vec<u8>) -> Result<(), ()>;
    fn send_hrmp_msg(from: u32, to: u32, msg: Vec<u8>) -> Result<(), ()>;
}

pub trait GetParaId {
	fn para_id() -> u32;
}
