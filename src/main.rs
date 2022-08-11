use num::Complex;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::RngCore;
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

fn main() {
    render_to_window().unwrap();
}

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

#[allow(dead_code)]
fn render_to_window() -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let mut input = WinitInputHelper::new();
    let window = {
        let size = PhysicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut fractal = Fractal::new(WIDTH as usize, HEIGHT as usize);

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            fractal.draw(pixels.get_frame());
            fractal.changed = false;
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::R) {
                fractal.update_colors();
            }

            if let Some(size) = input.window_resized() {
                // pixels.resize_buffer(size.width, size.height);
                pixels.resize_surface(size.width, size.height);
            }

            let scroll_delta = input.scroll_diff();
            if scroll_delta != 0.0 {
                if let Some(pos) = input.mouse() {
                    fractal.update_mouse(pos);
                }
                fractal.update_scroll(scroll_delta);
            }

            if input.mouse_held(0) {
                if input.mouse_diff() != (0.0, 0.0) {
                    fractal.update_mouse_drag(input.mouse_diff());
                }
            }

            if fractal.changed {
                window.request_redraw();
            }
        }
    });
}

struct Fractal {
    zoom: f32,
    centre: Complex<f64>,
    scaling_factor: f64,
    /// Width and Height in pixels of the picture
    bounds: (usize, usize),
    changed: bool,
    colors: Vec<[u8; 4]>,
}

const COLORS: usize = 8;

impl Fractal {
    fn new(width: usize, height: usize) -> Self {
        Self {
            zoom: 1.0,
            bounds: (width, height),
            centre: Complex { re: -0.75, im: 0.0 },
            scaling_factor: 10.0,
            changed: true,
            colors: get_colors(COLORS),
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.bounds.0) as f32;
            let y = (i / self.bounds.0) as f32;
            let point = self.pixel_to_point((x, y));

            let rgba = match escape_time(point, 256) {
                Some(t) => self.colors[t % COLORS],
                None => [0x00, 0x00, 0x00, 0xff],
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn pixel_to_point(&self, pixel: (f32, f32)) -> Complex<f64> {
        let scale = self.scaling_factor / (2.718_f32.powf(self.zoom)) as f64;
        let offset = scale / 2.0;
        Complex {
            re: (pixel.0 as f64 / self.bounds.0 as f64) * scale + (self.centre.re - offset),
            im: (pixel.1 as f64 / self.bounds.1 as f64) * scale + (self.centre.im - offset),
        }
    }

    fn update_scroll(&mut self, delta: f32) {
        self.changed = true;
        self.zoom = f32::max(1.0, self.zoom + delta);
    }

    fn update_mouse(&mut self, pos: (f32, f32)) {
        self.changed = true;
        self.centre = self.pixel_to_point(pos);
    }

    fn update_mouse_drag(&mut self, diff: (f32, f32)) {
        self.changed = true;
        self.centre = self.pixel_to_point((
            (self.bounds.0 as f32 / 2.0) - diff.0,
            (self.bounds.1 as f32 / 2.0) - diff.1,
        ));
    }

    fn update_colors(&mut self) {
        self.changed = true;
        self.colors = get_colors(COLORS);
    }
}

/// Returns the amount of iteration it takes z to escape the 2.0 circle given a c.
/// If limit is reached, z is assumed to not escape.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        } else {
            z = z * z + c;
        }
    }

    None
}

fn get_colors(n: usize) -> Vec<[u8; 4]> {
    let mut colors = Vec::with_capacity(n);
    for _ in 0..n {
        let mut color = [0u8; 4];
        rand::thread_rng().fill_bytes(&mut color);
        colors.push(color);
    }
    colors
}
