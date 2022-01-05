LED Matrix Clock
================

The project currently only supports [Adafruit RGB Matrix Bonnet](https://learn.adafruit.com/adafruit-rgb-matrix-bonnet-for-raspberry-pi), other adapters/connectors are not tested.

The LED Matrix panel needs to be Hub75 based, **WS2812 LEDs are not supported**, and given the situation there are lots of different panels using different chips and slightly different protocols, you may need to change the source code to use different chip model, scanning, mapping, addressing, etc.


Cross compiling
---------------
To cross compile the project on Linux host, you can use following command:
```
docker run \
--rm -it --user "$(id -u):$(id -g)" \
--env CC=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-cc \
--env CXX=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-c++ \
--env AR=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-ar \
-v "$HOME/.cargo":/root/.cargo \
-v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf \
cargo build --release
```

TODO:
-----
- [ ] Configurable LED panel size.
- [ ] More widgets.
- [ ] Text layout.
- [ ] And many other things.
