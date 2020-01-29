rm -rf tmp
mkdir -p tmp/lib64
cd $1 && cargo build --release && cd ..
cp target/release/$1 tmp/$1.x86_64
cp /usr/lib/libSDL2-2.0.so.0 tmp/
cp /usr/lib/libSDL2_gfx-1.0.so.0 tmp/
cp linux_runner.sh tmp/$1
chmod +x tmp/$1
butler push tmp liamoc/$2:linux
