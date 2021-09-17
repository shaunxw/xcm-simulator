# XCM Simulator

Test kit to simulate cross-chain message passing and XCM execution.

Different level of simulation are provided: `xcm-simulator` and `xcm-emulator`.

## xcm-simulator

**UPDATE**: `xcm-simulator` has been merged into [Polkadot repo](https://github.com/paritytech/polkadot/tree/master/xcm/xcm-simulator), and not actively maintained here anymore.

`xcm-simulator` uses *mock* relay chain and parachain runtime. It allows minimum runtime modules for XCM testing and playground, and thus has less compiling overhead depending on the specific config. Typical use case is unit tests of XCM related pallets.

## xcm-emulator

`xcm-emulator` uses production relay chain and parachain runtime. Users could plug in Kusama, Statemine, and Karura runtime etc. With up-to-date chain specs, it's able to verify if specific XCM messages work in live networks.

### Limitations

`xcm-emulator` emulates the delivery and execution of XCM messages, with the assumption that the message can always be delivered to and executed in destination. There are some reasons which could prevent messages being delivered or executed, such as:

- Number of messages in one block limitation a parachain can send is reached.
- There is no HRMP channel for messages.
- Relay chain run out of weights reserved for UMP messages execution.
- Parachain run out of weights reserved for XCMP/DMP messages execution.
- ...more possible reasons.

### Use cases

Typical use cases:
- Integration tests. Karura is using `xcm-emulator` for cross-chain transfer, homa-lite protocol integration tests.
- XCM message playground. Use it to verify if the message about to sent would work in live Kusama networks. Note the limitations in the above section.
