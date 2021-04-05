#[macro_export]
macro_rules! __construct_relay_chain_runtime {
	(
		pub mod $relay:ident {
			test_network = $test_network:path,
		}
	) => {
		pub mod $relay {
			use super::*;

			use $crate::traits::XcmRelay;

			$crate::frame_support::parameter_types! {
				pub const BlockHashCount: u64 = 250;
			}

			pub type AccountId = $crate::sp_runtime::AccountId32;

			impl $crate::frame_system::Config for Runtime {
				type Origin = Origin;
				type Call = Call;
				type Index = u64;
				type BlockNumber = u64;
				type Hash = $crate::sp_core::H256;
				type Hashing = $crate::sp_runtime::traits::BlakeTwo256;
				type AccountId = AccountId;
				type Lookup = $crate::sp_runtime::traits::IdentityLookup<Self::AccountId>;
				type Header = $crate::sp_runtime::testing::Header;
				type Event = Event;
				type BlockHashCount = BlockHashCount;
				type BlockWeights = ();
				type BlockLength = ();
				type Version = ();
				type PalletInfo = PalletInfo;
				type AccountData = $crate::pallet_balances::AccountData<Balance>;
				type OnNewAccount = ();
				type OnKilledAccount = ();
				type DbWeight = ();
				type BaseCallFilter = ();
				type SystemWeightInfo = ();
				type SS58Prefix = ();
				type OnSetCode = ();
			}

			pub type Balance = u128;

			$crate::frame_support::parameter_types! {
				pub const ExistentialDeposit: Balance = 0;
				pub const MaxLocks: u32 = 50;
			}

			impl $crate::pallet_balances::Config for Runtime {
				type Balance = Balance;
				type DustRemoval = ();
				type Event = Event;
				type ExistentialDeposit = ExistentialDeposit;
				type AccountStore = System;
				type MaxLocks = MaxLocks;
				type WeightInfo = ();
			}

			$crate::frame_support::parameter_types! {
				pub const RocLocation: $crate::xcm::v0::MultiLocation = $crate::xcm::v0::MultiLocation::Null;
				pub const RococoNetwork: $crate::xcm::v0::NetworkId = $crate::xcm::v0::NetworkId::Polkadot;
				pub const Ancestry: $crate::xcm::v0::MultiLocation = $crate::xcm::v0::MultiLocation::Null;
			}

			pub type LocationConverter = (
				$crate::xcm_builder::ChildParachainConvertsVia<$crate::cumulus_primitives_core::ParaId, AccountId>,
				$crate::xcm_builder::AccountId32Aliases<RococoNetwork, AccountId>,
			);

			pub type LocalAssetTransactor = $crate::xcm_builder::CurrencyAdapter<
				// Use this currency:
				Balances,
				// Use this currency when it is a fungible asset matching the given location or name:
				$crate::xcm_executor::traits::IsConcrete<RocLocation>,
				// We can convert the MultiLocations with our converter above:
				LocationConverter,
				// Our chain's account ID type (we can't get away without mentioning it explicitly):
				AccountId,
			>;

			type LocalOriginConverter = (
				$crate::xcm_builder::SovereignSignedViaLocation<LocationConverter, Origin>,
				$crate::xcm_builder::ChildParachainAsNative<$crate::runtime_parachains::origin::Origin, Origin>,
				$crate::xcm_builder::SignedAccountId32AsNative<RococoNetwork, Origin>,
				$crate::xcm_builder::ChildSystemParachainAsSuperuser<$crate::cumulus_primitives_core::ParaId, Origin>,
			);

			pub struct XcmSender;
			impl $crate::xcm::v0::SendXcm for XcmSender {
				fn send_xcm(dest: $crate::xcm::v0::MultiLocation, msg: $crate::xcm::v0::Xcm) -> $crate::xcm::v0::Result {
					use $crate::xcm::v0::{MultiLocation::*, Junction::*, Error};

					if let X2(Parent, Parachain { id }) = dest {
						<$test_network>::send_dmp_msg(id, msg)
					} else {
						Err(Error::CannotReachDestination)
					}
				}
			}

			pub struct XcmConfig;
			impl $crate::xcm_executor::Config for XcmConfig {
				type Call = Call;
				type XcmSender = ();
				type AssetTransactor = LocalAssetTransactor;
				type OriginConverter = LocalOriginConverter;
				type IsReserve = ();
				type IsTeleporter = ();
				type LocationInverter = $crate::xcm_builder::LocationInverter<Ancestry>;
			}

			impl $crate::runtime_parachains::origin::Config for Runtime {}

			pub type UmpSink = $crate::runtime_parachains::ump::XcmSink<XcmConfig>;

			type UncheckedExtrinsic = $crate::frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
			type Block = $crate::frame_system::mocking::MockBlock<Runtime>;

			use $crate::runtime_parachains::origin as runtime_parachains_origin;

			$crate::frame_support::construct_runtime!(
				pub enum Runtime where
					Block = Block,
					NodeBlock = Block,
					UncheckedExtrinsic = UncheckedExtrinsic,
				{
					//TODO: Use `$crate::frame_system` etc once `construct_runtime!` supports path.
					// https://github.com/paritytech/substrate/issues/8085
					System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
					Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
					ParachainsOrigin: runtime_parachains_origin::{Pallet, Origin},
				}
			);
		}
	}
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build<Runtime: frame_system::Config>(self) -> sp_io::TestExternalities
	where
		<Runtime as frame_system::Config>::BlockNumber: From<u64>,
	{
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| frame_system::Pallet::<Runtime>::set_block_number(1.into()));
		ext
	}
}

pub fn default_ext<Runtime: frame_system::Config>() -> sp_io::TestExternalities
where
	<Runtime as frame_system::Config>::BlockNumber: From<u64>,
{
	ExtBuilder::default().build::<Runtime>()
}
