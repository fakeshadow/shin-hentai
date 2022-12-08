# shin-hentai
a simple comic viewer

## Demo:
[WebAssembly](https://fakeshadow.github.io/)

## Requirement:
- Rust 1.65.0

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
3. the compiled binary is in `target/relase/` directory with the name `shin_hentai_bin`.

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

## Control:
- drag and drop zip file or folder to start viewing.
- `w` and `s` key for previous and next page.
- Mouse scroll can be used for navigate between page too.
