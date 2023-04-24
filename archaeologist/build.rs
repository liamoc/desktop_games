

use std::env;
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>> { 
    let target = env::var("TARGET").expect("TARGET was not set");
    if target.contains("windows") || target.contains("wasm32") {
    } else if target.contains("linux") ||
       target.contains("dragonfly") ||
       target.contains("freebsd") ||
       target.contains("netbsd") ||
       target.contains("openbsd") {
    } else {
        println!("cargo:rustc-link-lib=framework=SDL2");
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
        println!("cargo:rustc-link-search=framework=/Users/liamoc/Library/Frameworks");
    }
    Ok(())
}
