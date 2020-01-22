fn main() {
    println!("cargo:rustc-link-lib=framework=SDL2");
    println!("cargo:rustc-link-lib=framework=SDL2_gfx");
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
    println!("cargo:rustc-link-search=framework=/Library/Frameworks");
}