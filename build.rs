use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").expect("TARGET is not set.");
    Command::new("make")
        .args(&["-C", "rpi-rgb-led-matrix/lib"])
        .env("CC", format!("{}-gcc", target))
        .env("CXX", format!("{}-g++", target))
        .env("AR", format!("{}-ar", target))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native=rpi-rgb-led-matrix/lib");
}
