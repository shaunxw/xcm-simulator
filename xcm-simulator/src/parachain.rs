use sp_io::TestExternalities;

#[macro_export]
macro_rules! __construct_parachain_runtime {
	// entry point with default xcm config
	(
		pub mod $para:ident {
			test_network = $test_network:path,
			xcm_config = { default },
			extra_config = {
				$( $extra_config:tt )*
			},
			extra_modules = {
				$( $extra_modules:tt )*
			},
		}
	) => {
		$crate::__construct_parachain_runtime! {
			pub mod $para {
				test_network = $test_network,
				xcm_config = {
					$crate::frame_support::parameter_types! {
						pub Network: $crate::xcm::v0::NetworkId = $crate::xcm::v0::NetworkId::Any;
						pub RelayChainOrigin: Origin = Into::<Origin>::into($crate::cumulus_pallet_xcm::Origin::Relay);
						pub Ancestry: $crate::xcm::v0::MultiLocation = $crate::xcm::v0::MultiLocation::X1(
							$crate::xcm::v0::Junction::Parachain {
								id: <ParachainInfo as $crate::frame_support::traits::Get<_>>::get().into(),
							}
						);
					}

					pub type LocationConverter = (
						$crate::xcm_builder::ParentIsDefault<AccountId>,
						$crate::xcm_builder::SiblingParachainConvertsVia<$crate::polkadot_parachain::primitives::Sibling, AccountId>,
						$crate::xcm_builder::AccountId32Aliases<Network, AccountId>,
					);

					pub type LocalOriginConverter = (
						$crate::xcm_builder::SovereignSignedViaLocation<LocationConverter, Origin>,
						$crate::xcm_builder::RelayChainAsNative<RelayChainOrigin, Origin>,
						$crate::xcm_builder::SiblingParachainAsNative<$crate::cumulus_pallet_xcm::Origin, Origin>,
						$crate::xcm_builder::SignedAccountId32AsNative<Network, Origin>,
					);

					pub struct XcmSender;
					impl $crate::xcm::v0::SendXcm for XcmSender {
						fn send_xcm(dest: $crate::xcm::v0::MultiLocation, msg: $crate::xcm::v0::opaque::Xcm) -> $crate::xcm::v0::Result {
							use $crate::xcm::v0::{MultiLocation::*, Junction::*, Error};

							if let X1(Parachain { id }) = dest {
								<$test_network>::send_dmp_msg(id, msg)
							} else {
								Err(Error::CannotReachDestination(dest, msg))
							}
						}
					}

					$crate::frame_support::parameter_types! {
						pub UnitWeightCost: $crate::frame_support::weights::Weight = 1_000;
						pub const WeightPrice: ($crate::xcm::v0::MultiLocation, u128) = ($crate::xcm::v0::MultiLocation::X1($crate::xcm::v0::Junction::Parent), 1_000);
					}

					pub struct XcmConfig;
					impl $crate::xcm_executor::Config for XcmConfig {
						type Call = Call;
						type XcmSender = XcmSender;
						type AssetTransactor = ();
						type OriginConverter = LocalOriginConverter;
						type IsReserve = $crate::xcm_builder::NativeAsset;
						type IsTeleporter = ();
						type LocationInverter = $crate::xcm_builder::LocationInverter<Ancestry>;
						type Barrier = ();
						type Weigher = $crate::xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
						type Trader = $crate::xcm_builder::FixedRateOfConcreteFungible<WeightPrice>;
						type ResponseHandler = ();
					}
				},
				extra_config = {
					$( $extra_config )*
				},
				extra_modules = {
					$( $extra_modules )*
				},
			}
		}
	};

	// entry point with customized xcm config
	(
		pub mod $para:ident {
			test_network = $test_network:path,
			xcm_config = { $( $xcm_config:tt )* },
			extra_config = { $( $extra_config:tt )* },
			extra_modules = { $( $extra_modules:tt )* },
		}
	) => {
		pub mod $para {
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
				type AccountData = ();
				type OnNewAccount = ();
				type OnKilledAccount = ();
				type DbWeight = ();
				type BaseCallFilter = ();
				type SystemWeightInfo = ();
				type SS58Prefix = ();
				type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
			}

			impl $crate::parachain_info::Config for Runtime {}

			pub struct MockMessenger;
			impl $crate::cumulus_primitives_core::UpwardMessageSender for MockMessenger {
				fn send_upward_message(
					msg: $crate::polkadot_parachain::primitives::UpwardMessage
				) -> Result<u32, $crate::cumulus_primitives_core::MessageSendError> {
					let _ = <$test_network>::send_ump_msg(ParachainInfo::parachain_id().into(), msg);
					Ok(0)
				}
			}

			impl $crate::cumulus_primitives_core::XcmpMessageSource for MockMessenger {
				fn take_outbound_messages(
					_maximum_channels: usize
				) -> Vec<($crate::cumulus_primitives_core::ParaId, Vec<u8>)> { vec![] }
			}

			impl $crate::cumulus_primitives_core::XcmpMessageHandler for MockMessenger {
				fn handle_xcmp_messages<'a, I: Iterator<Item=($crate::cumulus_primitives_core::ParaId, $crate::polkadot_core_primitives::BlockNumber, &'a [u8])>>(
					iter: I,
					_max_weight: $crate::frame_support::weights::Weight,
				) -> $crate::frame_support::weights::Weight { for _ in iter {} 0 }
			}

			impl $crate::cumulus_primitives_core::DownwardMessageHandler for MockMessenger {
				fn handle_downward_message(_msg: $crate::cumulus_primitives_core::InboundDownwardMessage) -> $crate::frame_support::weights::Weight { 0 }
			}

			$( $xcm_config )*

			impl $crate::cumulus_pallet_xcm::Config for Runtime {}

			impl $crate::cumulus_pallet_parachain_system::Config for Runtime {
				type Event = Event;
				type OnValidationData = ();
				type SelfParaId = parachain_info::Module<Runtime>;
				type DownwardMessageHandlers = MockMessenger;
				type OutboundXcmpMessageSource = MockMessenger;
				type XcmpMessageHandler = MockMessenger;
				type ReservedXcmpWeight = ();
			}

			$( $extra_config )*

			type UncheckedExtrinsic = $crate::frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
			type Block = $crate::frame_system::mocking::MockBlock<Runtime>;

			$crate::frame_support::construct_runtime!(
				pub enum Runtime where
					Block = Block,
					NodeBlock = Block,
					UncheckedExtrinsic = UncheckedExtrinsic,
				{
					//TODO: Use `$crate::frame_system` etc once `construct_runtime!` supports path.
					// https://github.com/paritytech/substrate/issues/8085
					System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
					ParachainInfo: parachain_info::{Pallet, Storage, Config},
					XcmHandler: cumulus_pallet_xcm::{Pallet, Origin},
					ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
					$( $extra_modules )*
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
	pub fn build<Runtime: frame_system::Config>(self, para_id: u32) -> TestExternalities
	where
		<Runtime as frame_system::Config>::BlockNumber: From<u64>,
	{
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		parachain_info::GenesisConfig {
			parachain_id: para_id.into(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = TestExternalities::new(t);
		ext.execute_with(|| frame_system::Pallet::<Runtime>::set_block_number(1.into()));
		ext
	}
}

pub fn default_ext<Runtime: frame_system::Config>(para_id: u32) -> TestExternalities
where
	<Runtime as frame_system::Config>::BlockNumber: From<u64>,
{
	ExtBuilder::default().build::<Runtime>(para_id)
}