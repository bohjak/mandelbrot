# Mandelbrot Set Explorer

Interactive (as in one can zoom and move around) Mandelbrot set viewer. GPU-rendered with [wgpu](https://wgpu.rs).

## How To Run

**Prerequisites:** [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (rustc version >= 1.56.0), for WASM build [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), and for dev server [go](https://go.dev/dl/).

It's possible to run either natively or in the browser.

First step is to clone the repository.

```bash
git clone https://github.com/bohjak/mandelbrot
cd mandelbrot
```

For native build simply run release cargo build.

```bash
cargo run --release
```

For running in the browser:

1. build with wasm-pack;
   ```bash
   wasm-pack build --target web
   ```
1. run server;
   ```bash
   go run server.go
   ```
1. open in browser.
   ```bash
   open http://localhost:3000
   ```

> **Note**  
> The server uses server-sent events to reload the page when receiving a request to /sse/reload. It's possible to automatically build the project and reload the page for development convenience.  
> E.g. with [entr](https://eradman.com/entrproject/): `ls src/*.rs | entr -cs 'wasm-pack build --dev --target=web && curl http://localhost:3000/sse/reload'`

## Controls

- **Mouse:** left-click and drag to move around, scroll-wheel to zoom;
- **ArrowLeft / H / A:** pan left;
- **ArrowDown / J / S:** pan down;
- **ArrowUp / K / W:** pan up;
- **ArrowRight / L / D:** pan right;
- **Ctrl + Direction:** pan faster;
- **Shift + Direction:** pan slower;
- **Plus / Equals:** zoom in;
- **Minus:** zoom out;
- **R:** reset position and zoom level.
