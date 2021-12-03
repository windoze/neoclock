use std::process::Command;

fn main() {
    Command::new("make")
        .args(&["-C", "rpi-rgb-led-matrix/lib"])
        .env("CC", "armv7-unknown-linux-gnueabihf-gcc")
        .env("CXX", "armv7-unknown-linux-gnueabihf-g++")
        .env("AR", "armv7-unknown-linux-gnueabihf-ar")
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", "rpi-rgb-led-matrix/lib");
}
