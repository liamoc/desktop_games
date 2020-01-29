# Micro Entertainment Pack

This is the source code to the Micro Entertainment Pack, a collection of tiny desktop games I made for fun. If you're just interested in playing the games, probably better head over to [my itch.io page](http://liamoc.itch.io). I try to maintain builds for Linux, Windows, and macOS but macOS is the easiest for me to support.

All games are made using Rust and my [tesserae](https://crates.io/crates/tesserae/) library for graphics composed out of 8x8 2 colour tiles. All of the games in the pack use a common tileset to do all of their graphical drawing. All graphics were drawn using the tesseraed editor that comes with the library. I also used this tileset to build the icons and cover art for the game, along with effects from Affinity Photo.

For input handling, rendering of graphics, and regulation of framerate, I use the cross-platform SDL2 and SDL2\_gfx libraries.

# Building


*The makefiles and build scripts in this repository are for building bundles and deploying them to itch.io*. They are not for building the games themselves. To do that, you can just use cargo, like any other Rust project.

You will need SDL2 and SDL2\_gfx installed on your system in some form or other. Rust's sdl2 crate does support bundled SDL2 but I haven't investigated this as it's not needed for my use case.

Obviously you will also need rustc and cargo, I recommend installing it via rustup.

Cargo should build binaries out of the box just by running cargo build. 

# Contributing

These games are all open source, with code under BSD3 and the icons and tile set under CC0. I naturally welcome contributions, *HOWEVER*:

- These games were all made quickly and simply. Often they're all just in one file. I acknowledge that the code may not be the most elegant or beautiful, but it also doesn't have to be. Criticism of programming style will be ignored. 
- I am loathe to increase the complexity of these games significantly. Adding new features will be done cautiously if at all.
- Any pull request that increases the dynamic library dependency footprint of the games will be rejected. The more dependencies, the higher the maintenance burden. Please keep that in mind.
- I am not a big believer in codes of conduct or other moral formalism, but I do expect contributors to meet a level of standard decency: not being a mysogynist, nazi, white supremacist, transphobe or homophobe is a good start. I will refuse contributions from people who fall short of these standards.
