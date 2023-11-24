# shin-hentai
a simple comic viewer

## Demo:
[WebAssembly](https://fakeshadow.github.io/)

## Requirement:
- Rust 1.70

## Supported platform:
- Linux
- macOS
- Windows
- Web (Partial functional)

## Build targeting desktop
1. install Rust language. Click [here](https://www.rust-lang.org/learn/get-started) to see how.
2. clone and compile the project
    ```commandline
    git clone https://github.com/fakeshadow/shin-hentai
    cd shin-hentai
    cargo build --release
    ```
3. the compiled binary is in `target/release/` directory with the name `shin_hentai_bin`.

## Build targeting web
1. install Rust language. Click [here](https://www.rust-lang.org/learn/get-started) to see how.
2. install Trunk. Click [here](https://trunkrs.dev/#install) to see how.
3. (Optional) serve project locally
    ```commandline
    trunk serve
    ```
   Open http://127.0.0.1:8080/index.html#dev in a browser
4. build the project
   ```commandline
   trunk build --release
   ```
5. the compiled static files are in `dist` directory.

## Build targeting wayland on linux
1. Reference building process targeting desktop and use `cargo build --release --features wayland` instead of `cargo build --release`
2. setup application file and icons. [reference](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#desktop-file-id). 
3. Example:
```
location for desktop file:
/usr/share/applications/shin_hentai_bin.desktop

sample entry for desktop file:
[Desktop Entry]
Version=1.0
Name=maji_hentai
Exec=shin_hentai_bin
Icon=shin-hentai
Terminal=false
Type=Application

location for application icon:
/usr/share/icons/hicolor/scalable/apps/

example of copy shin-hentai icon:
sudo cp ~/shin-hentai/resource/shin-hentai.svg /usr/share/icons/hicolor/scalable/apps/  
```

## Control:
- drag and drop zip file or folder to start viewing.
- `w` and `s` key for previous and next page.
- `ctrl + w` to first and `ctrl + s` to last page.
- Mouse scroll can be used for navigate between page too.
