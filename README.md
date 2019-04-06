## peach-buttons

GPIO microservice module for handling button presses. Each button press results in a JSON-RPC client call to the `peach-menu` microservice. A button code for the pressed button is sent with the call to `peach-menu`, allowing state-specific actions to be taken.

_Note: This module is a work-in-progress. Buttons need debouncing._

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

### Licensing

AGPL-3.0
