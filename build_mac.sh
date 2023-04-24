set -x
cd $1 && cargo bundle --release --target x86_64-apple-darwin && cargo bundle --release && cd ..
#cd "target/release/bundle/osx/$3.app/Contents/MacOS" && install_name_tool -add_rpath "@executable_path/../Frameworks" $1 && cd -
mkdir -p "target/release/bundle/universal"
rm -rf "target/release/bundle/universal/$3.app"
cp -R "target/release/bundle/osx/$3.app" "target/release/bundle/universal/$3.app"
rm -rf "target/release/bundle/universal/$3.app/Contents/MacOS/$1"
lipo -create "target/x86_64-apple-darwin/release/bundle/osx/$3.app/Contents/MacOS/$1" "target/release/bundle/osx/$3.app/Contents/MacOS/$1" -output  "target/release/bundle/universal/$3.app/Contents/MacOS/$1"
butler push "target/release/bundle/universal/$3.app" liamoc/$2:osx
