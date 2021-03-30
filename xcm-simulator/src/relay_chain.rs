use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32};

use polkadot_parachain::primitives::Id as ParaId;
use xcm::v0::{MultiLocation, NetworkId};
use xcm_builder::{
	AccountId32Aliases, ChildParachainAsNative, ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
	CurrencyAdapter as XcmCurrencyAdapter, LocationInverter, SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::traits::IsConcrete;

pub mod default {
	use super::*;

	use runtime_parachains::origin as parachains_origin;
	use runtime_parachains::ump as parachains_ump;

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
	}

	pub type AccountId = AccountId32;

	impl frame_system::Config for Runtime {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = ::sp_runtime::traits::BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type BlockWeights = ();
		type BlockLength = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
	}

	pub type Balance = u128;

	parameter_types! {
		pub const ExistentialDeposit: Balance = 0;
		pub const MaxLocks: u32 = 50;
	}

	impl pallet_balances::Config for Runtime {
		type Balance = Balance;
		type DustRemoval = ();
		type Event = Event;
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type MaxLocks = MaxLocks;
		type WeightInfo = ();
	}

	parameter_types! {
		pub const RocLocation: MultiLocation = MultiLocation::Null;
		pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
		pub const Ancestry: MultiLocation = MultiLocation::Null;
	}

	pub type LocationConverter = (
		ChildParachainConvertsVia<ParaId, AccountId>,
		AccountId32Aliases<RococoNetwork, AccountId>,
	);

	pub type LocalAssetTransactor = XcmCurrencyAdapter<
		// Use this currency:
		Balances,
		// Use this currency when it is a fungible asset matching the given location or name:
		IsConcrete<RocLocation>,
		// We can convert the MultiLocations with our converter above:
		LocationConverter,
		// Our chain's account ID type (we can't get away without mentioning it explicitly):
		AccountId,
	>;

	type LocalOriginConverter = (
		SovereignSignedViaLocation<LocationConverter, Origin>,
		ChildParachainAsNative<parachains_origin::Origin, Origin>,
		SignedAccountId32AsNative<RococoNetwork, Origin>,
		ChildSystemParachainAsSuperuser<ParaId, Origin>,
	);

	pub struct XcmConfig;
	impl xcm_executor::Config for XcmConfig {
		type Call = Call;
		type XcmSender = ();
		type AssetTransactor = LocalAssetTransactor;
		type OriginConverter = LocalOriginConverter;
		type IsReserve = ();
		type IsTeleporter = ();
		type LocationInverter = LocationInverter<Ancestry>;
	}

	impl parachains_origin::Config for Runtime {}

	pub type UmpSink = parachains_ump::XcmSink<XcmConfig>;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
	type Block = frame_system::mocking::MockBlock<Runtime>;

	frame_support::construct_runtime!(
		pub enum Runtime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			ParachainsOrigin: parachains_origin::{Pallet, Origin},
		}
	);

	pub struct ExtBuilder;

	impl Default for ExtBuilder {
		fn default() -> Self {
			ExtBuilder
		}
	}

	impl ExtBuilder {
		pub fn build(self) -> sp_io::TestExternalities {
			let t = frame_system::GenesisConfig::default()
				.build_storage::<Runtime>()
				.unwrap();

			let mut ext = sp_io::TestExternalities::new(t);
			ext.execute_with(|| frame_system::Pallet::<Runtime>::set_block_number(1));
			ext
		}
	}

	pub fn ext() -> sp_io::TestExternalities {
		ExtBuilder::default().build()
	}
}
