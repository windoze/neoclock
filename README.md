LED Matrix Clock
================

The project currently only supports [Adafruit RGB Matrix Bonnet](https://learn.adafruit.com/adafruit-rgb-matrix-bonnet-for-raspberry-pi), other adapters/connectors are not tested.


The LED Matrix panel needs to be Hub75 based, **WS2812 LEDs are not supported**, and given the situation there are lots of different panels using different chips and slightly different protocols, you may need to change the source code to use different chip model, scanning, mapping, addressing, etc.

In-door panels are preferred, out-door models are much brighter but they need larger current and may use much more complicated addressing and scanning modes.

NOTE: The program currently is hard-coded to use 64x64 panel, check out [AliExpress](https://www.aliexpress.com/wholesale?catId=0&initiative_id=SB_20220125075658&SearchText=hub75+64x64+led) to find panels.

![It's Rinning!](/assets/images/neoclock.gif "It's running!")

Build the simulator
-------------------
Use `cargo build` to build **the simulator** which runs on host.

Build on Raspberry Pi
---------------------
Or you can run `cargo build --release --features rpi --no-default-features` to build the executable on RPi directly, but it could take half an hour or even longer due to the slow I/O.

Cross compiling
---------------
To cross compile the project on Linux (include WSL) host with Docker installed, you can use following command:
```
docker run \
--rm -it --user "$(id -u):$(id -g)" \
--env CC=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-cc \
--env CXX=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-c++ \
--env AR=/usr/local/musl/bin/armv7-unknown-linux-musleabihf-ar \
-v "$HOME/.cargo":/root/.cargo \
-v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf \
cargo build --release --features rpi --no-default-features
```

Run
---

Before running the program, several preparations need to be done:
- First, you need a stable 5V power supply which can provide at least 4A current, 10A is preferred, LED panels consume large current, especially when mose LEDs are lit up. Each pixel can draw up to 0.06 Amps each if on full white, do you own math.
- To use 64 rows panel, you need to solder a jumper on Adafruit RGB Matrix Bonnet. <img src="https://cdn-learn.adafruit.com/assets/assets/000/063/007/medium640/led_matrices_addr-e-pad-bonnet.jpg?1538677462" />
- (Optional) Then you need to solder a jump between GPIO4 and GPIO8 on the bonnet to use the hardware PWM.<img src="https://cdn-learn.adafruit.com/assets/assets/000/057/727/medium640/led_matrices_gpios.jpg?1531951340" />
Otherwise the graphics on the LED panel may flicker or distort.
- If you've done the steps above, you also need to switch off on-board sound (dtparam=audio=off in /boot/config.txt). You can still use external USB sound adaptors to play audios.
- You need to set up a MQTT server somewhere that the RPi can connect to, and set following 3 environment variables:
    + `NEOCLOCK_HOSTNAME`, the MQTT server host name.
    + `NEOCLOCK_DEVICE_ID`, the MQTT device id for NeoClock.
    + `NEOCLOCK_PASSWORD`, the password to be used to connect to the MQTT server.

    Without a MQTT server, all dynamic features are **not** available, you'll have to use config file to set up all the components.

    NOTE: The Azure IoT Hub doesn't work as it only supports a subset of MQTT protocol.

You need to run the program as `root` to enable hardware PWM, otherwise only software PWM will be used and the flickering may appear.

To start the program, run `sudo -E /path/to/neoclock` to inherit the environment from the current user.

TODO:
-----
- [ ] Configurable LED panel size.
- [ ] More widgets.
- [ ] Text layout.
- [ ] And many other things.
