# Mandelbrot Set Explorer

Interactive (as in one can zoom and move around) Mandelbrot set viewer. GPU-rendered with [wgpu](https://wgpu.rs).

## How To Run

**Prerequisites:** [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (rustc version >= 1.56.0)

1. Clone the repository
   ```bash
   git clone https://github.com/bohjak/mandelbrot
   cd mandelbrot
   ```
1. Run release build
   ```bash
   cargo run --release
   ```

## Controls

- **Pan:** hold left mouse button and drag;
- **Zoom:** scroll - up to zoom in, down to zoom out;
- **Quit:** escape.
