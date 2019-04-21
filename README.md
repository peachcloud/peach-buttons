## peach-buttons

GPIO microservice module for handling button presses. `peach-buttons` implements a JSON-RPC server with [Publish-Subscribe extension](https://docs.rs/jsonrpc-pubsub/11.0.0/jsonrpc_pubsub/). Each button press results in a JSON-RPC request being sent over websockets to any subscribers. A button code for the pressed button is sent with the request to subscribers, allowing state-specific actions to be taken.

In the intended implementation of PeachCloud, `peach-menu` will subscribe to `peach-buttons` events.

_Note: This module is a work-in-progress._

### Pin to Button to Button Code Mappings

```
462 => Center => 0,
485 => Left => 1,
481 => Right => 2,
475 => Up => 3,
480 => Down => 4,
463 => A => 5,
464 => B => 6
```

_Note: Pin numbers are offset by 458 for Debian on RPi3._

### Setup

Clone this repo:

`git clone https://github.com/peachcloud/peach-buttons.git`

Move into the repo and compile:

`cd peach-buttons`  
`cargo build`

Run the binary with sudo:

`sudo ./target/debug/peach-buttons`

### Testing Subscription

Request:
  
`{"id":1,"jsonrpc":"2.0","method":"subscribe_buttons"}`

Response:

`{"jsonrpc":"2.0","result":1,"id":1}`

Event:

`{"jsonrpc":"2.0","method":"button_press","params":[0]}`

### Licensing

AGPL-3.0
