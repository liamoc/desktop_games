rm -rf tmp
mkdir -p tmp
cd $1 && cargo build --release && cd ..
cp target/release/$1.exe tmp/
cp /mingw64/bin/libSDL2_* tmp/
cp /mingw64/bin/SDL2* tmp/
cp $1/icon_windows.ico tmp/ && cd tmp && "C:\Program Files (x86)\Resource Hacker\ResourceHacker.exe" -open $1.exe -save $1.exe -action add -resource icon_windows.ico -mask ICONGROUP,MAINICON,0 && rm icon_windows.ico && cd ..
butler push tmp liamoc/$2:windows
