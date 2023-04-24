rm -rf tmp
mkdir -p tmp
cd $1 && cargo build --target=x86_64-pc-windows-gnu --release && cd ..
cp target/x86_64-pc-windows-gnu/release/$1.exe tmp/
cp ~/misc/windows_stuff/*.dll tmp/
cp $1/icon_windows.ico tmp/ && cd tmp && wine64 ~/misc/windows_stuff/rcedit-x64.exe --set-icon icon_windows.ico $1.exe && rm icon_windows.ico && cd ..
butler push tmp liamoc/$2:windows
