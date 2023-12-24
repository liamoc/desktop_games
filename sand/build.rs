
use std::env;

fn main() {
    let target = env::var("TARGET").expect("TARGET was not set");
    if target.contains("windows") {
        //println!("cargo:rustc-link-search=D:\\msys64\\mingw64\\lib");
    } else if target.contains("linux") ||
       target.contains("dragonfly") ||
       target.contains("freebsd") ||
       target.contains("netbsd") ||
       target.contains("openbsd") {
    } else {
        println!("cargo:rustc-link-lib=framework=SDL2");
        //println!("cargo:rustc-link-lib=framework=SDL2_gfx");
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
        println!("cargo:rustc-link-search=framework=/Users/liamoc/Library/Frameworks");

    }
}
