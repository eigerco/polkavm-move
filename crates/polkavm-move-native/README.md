# Move native library

This library contains the 'host' and 'guest' bits for use with the [Move to PolkaVM project](https://github.com/eigerco/polkavm-move).

In this context, 'host' means the machine running the PolkaVM, and 'guest' means the code running
_inside_ the PVM.

It contains the code to implement the Move stdlib native functions.

#### Note

All target/toolchain configs are taken from [polkaVM repo example program](https://github.com/paritytech/polkavm/tree/master/guest-programs/example-hello-world)
