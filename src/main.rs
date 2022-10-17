use mandelbrot::run;

fn main() {
    pollster::block_on(run());
}
