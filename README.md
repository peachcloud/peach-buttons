## peach-buttons

GPIO microservice module for handling button presses. `peach-buttons` implements a JSON-RPC server with [Publish-Subscribe extension](https://docs.rs/jsonrpc-pubsub/11.0.0/jsonrpc_pubsub/). Each button press results in a JSON-RPC request being sent over websockets to any subscribers. A button code for the pressed button is sent with the request to subscribers, allowing state-specific actions to be taken.

In the intended implementation of PeachCloud, `peach-menu` will subscribe to `peach-buttons` events.

_Note: This module is a work-in-progress._

### Pin to Button to Button Code Mappings

```
4 => Center => 0,
27 => Left => 1,
23 => Right => 2,
17 => Up => 3,
22 => Down => 4,
5 => A => 5,
6 => B => 6
```

_Note: `peach-buttons` utilizes the GPIO character device ABI. This API, stabilized with Linux v4.4, deprecates the legacy sysfs interface to GPIOs that is planned to be removed from the upstream kernel after year 2020._

### Setup

Clone this repo:

`git clone https://github.com/peachcloud/peach-buttons.git`

Move into the repo and compile:

`cd peach-buttons`  
`cargo build`

Run the binary with sudo:

`sudo ./target/debug/peach-buttons`

Logging is also made availabe with `env_logger`:

`sudo RUST_LOG=info ./target/debug/peach-buttons`

_Other logging levels include debug, warn and error._

### Testing Subscription

Request:
  
`{"id":1,"jsonrpc":"2.0","method":"subscribe_buttons"}`

Response:

`{"jsonrpc":"2.0","result":1,"id":1}`

Event:

`{"jsonrpc":"2.0","method":"button_press","params":[0]}`

### Licensing

AGPL-3.0
