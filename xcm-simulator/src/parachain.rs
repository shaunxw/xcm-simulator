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
                        pub RelayChainOrigin: Origin = Into::<Origin>::into($crate::cumulus_pallet_xcm_handler::Origin::Relay);
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
                        $crate::xcm_builder::SiblingParachainAsNative<$crate::cumulus_pallet_xcm_handler::Origin, Origin>,
                        $crate::xcm_builder::SignedAccountId32AsNative<Network, Origin>,
                    );

                    pub struct XcmConfig;
                    impl $crate::xcm_executor::Config for XcmConfig {
                        type Call = Call;
                        type XcmSender = XcmHandler;
                        type AssetTransactor = ();
                        type OriginConverter = LocalOriginConverter;
                        type IsReserve = $crate::xcm_executor::traits::NativeAsset;
                        type IsTeleporter = ();
                        type LocationInverter = $crate::xcm_builder::LocationInverter<Ancestry>;
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
            }

            impl $crate::parachain_info::Config for Runtime {}

            pub struct MockMessenger;
            impl $crate::cumulus_primitives_core::UpwardMessageSender for MockMessenger {
                fn send_upward_message(msg: $crate::polkadot_parachain::primitives::UpwardMessage) -> Result<(), ()> {
                    <$test_network>::send_ump_msg(ParachainInfo::parachain_id().into(), msg)
                }
            }

            impl $crate::cumulus_primitives_core::HrmpMessageSender for MockMessenger {
                fn send_hrmp_message(msg: $crate::cumulus_primitives_core::OutboundHrmpMessage) -> Result<(), ()> {
                    let $crate::cumulus_primitives_core::OutboundHrmpMessage { recipient, data } = msg;
                    <$test_network>::send_hrmp_msg(ParachainInfo::parachain_id().into(), recipient.into(), data)
                }
            }

            $( $xcm_config )*

            impl $crate::cumulus_pallet_xcm_handler::Config for Runtime {
                type Event = Event;
                type XcmExecutor = $crate::xcm_executor::XcmExecutor<XcmConfig>;
                type UpwardMessageSender = MockMessenger;
                type HrmpMessageSender = MockMessenger;
                type SendXcmOrigin = $crate::frame_system::EnsureRoot<AccountId>;
                type AccountIdConverter = LocationConverter;
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
                    XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Call, Event<T>, Origin},

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
