LED Matrix Clock
================

The project currently only supports [Adafruit RGB Matrix Bonnet](https://learn.adafruit.com/adafruit-rgb-matrix-bonnet-for-raspberry-pi), other adapters/connectors are not tested.

The project is configured to use [messense/macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains) on macOS, the paths of GCC tools need to be changed if you're using other distribution of ARM GCC toolchain.

The LED Matrix panel needs to be Hub75 based, **WS2812 LEDs are not supported**, and given the situation there are lots of different panels using different chips and slightly different protocols, you may need to change the source code to use differnt chip model, scanning, mapping, addressing, etc.

TODO:
-----

* Configurable LED panel size.
* More widgets.
* Full font support for clock and calendar widgets.
* And many other things.