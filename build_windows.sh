rm -rf tmp
mkdir tmp
cd $1 && cargo build --release && cd ..
cp target/release/$1.exe tmp/
cp /mingw64/bin/libSDL2_* tmp/
cp /mingw64/bin/SDL2* tmp/
butler push tmp liamoc/$2:windows
