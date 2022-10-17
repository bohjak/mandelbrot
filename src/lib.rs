use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};
use winit::{
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

pub async fn run() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = WindowBuilder::new()
        .with_title("Mandelbrot")
        .with_inner_size(winit::dpi::PhysicalSize::new(1600.0, 1600.0))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            state.update_viewport();
            match state.render() {
                Ok(_) => (),
                // Recorfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // Quit if system is out of memory
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // Outdated and Timeout should resolve themselves by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }

        if input.update(&event) {
            if input.quit() || input.key_pressed(VirtualKeyCode::Escape) {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                state.resize(size);
            }

            if input.scroll_diff() != 0.0 {
                if let Some(pos) = input.mouse() {
                    state.viewport.set_centre(pos);
                }
                state.viewport.update_zoom(input.scroll_diff());
            }

            if input.mouse_held(0) {
                if input.mouse_diff() != (0.0, 0.0) {
                    state.viewport.move_centre(input.mouse_diff());
                }
            }

            window.request_redraw();
        }
    })
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const SQUARE: &[Vertex] = &[
    // NW, SW, SE
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    // NW, SE, NE
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
];

struct Viewport {
    /// Zoom level; >= 0.0
    zoom: f32,
    /// Window width in physical pixels
    pixel_width: f32,
    /// Window height in physical pixels
    pixel_height: f32,
    /// Viewport width on the complex plane
    point_width: f32,
    /// Offset from the complex plane origin
    centre: [f32; 2],
}

impl Viewport {
    fn new(window: PhysicalSize<u32>) -> Self {
        Self {
            zoom: 0.0,
            pixel_width: window.width as f32,
            pixel_height: window.height as f32,
            point_width: 4.0,
            centre: [0.0, 0.0],
        }
    }

    fn update_window_size(&mut self, window: PhysicalSize<u32>) {
        self.pixel_width = window.width as f32;
    }

    fn move_centre(&mut self, delta: (f32, f32)) {
        let scale = self.scale();
        self.centre[0] -= delta.0 * scale;
        self.centre[1] -= delta.1 * scale;
    }

    fn set_centre(&mut self, pos: (f32, f32)) {
        let scale = self.scale();
        self.centre[0] += (pos.0 - self.pixel_width / 2.0) * scale;
        self.centre[1] += (pos.1 - self.pixel_height / 2.0) * scale;
    }

    fn update_zoom(&mut self, delta: f32) {
        let new_zoom = self.zoom + delta;
        self.zoom = if new_zoom < 0.0 { 0.0 } else { new_zoom };
    }

    fn scale(&self) -> f32 {
        return (1.0 / 2f32.powf(self.zoom)) * (self.point_width / self.pixel_width);
    }
}

/// Stores information about viewport and the complex plane position
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    /// Ratio of points per pixel
    scale: f32,
    /// Offset from the complex plane origin on the x axis
    cx: f32,
    /// Offset from the complex plane origin on the y axis
    cy: f32,
    /// Middle of the x axis in pixels
    xoff: f32,
    /// Middle of the y axis in pixels
    yoff: f32,
}

impl ViewportUniform {
    fn new(viewport: &Viewport) -> Self {
        Self {
            scale: viewport.scale(),
            cx: viewport.centre[0],
            cy: viewport.centre[1],
            xoff: viewport.pixel_width / 2.0,
            yoff: viewport.pixel_height / 2.0,
        }
    }

    fn update_viewport(&mut self, viewport: &Viewport) {
        self.scale = viewport.scale();
        self.cx = viewport.centre[0];
        self.cy = viewport.centre[1];
        self.xoff = viewport.pixel_width / 2.0;
        self.yoff = viewport.pixel_height / 2.0;
    }
}

struct State {
    size: PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    viewport: Viewport,
    viewport_uniform: ViewportUniform,
    viewport_buffer: wgpu::Buffer,
    viewport_bind_group: wgpu::BindGroup,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        assert_ne!(size.width, 0);
        assert_ne!(size.height, 0);

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let viewport = Viewport::new(size);
        let viewport_uniform = ViewportUniform::new(&viewport);
        let viewport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport Buffer"),
            contents: bytemuck::cast_slice(&[viewport_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &viewport_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&viewport_bind_group_layout],
                push_constant_ranges: &[],
                label: None,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            layout: Some(&render_pipeline_layout),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            depth_stencil: None,
            label: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(SQUARE),
            label: None,
        });

        let num_vertices = SQUARE.len() as u32;

        Self {
            size,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            viewport,
            viewport_uniform,
            viewport_buffer,
            viewport_bind_group,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.viewport.update_window_size(new_size);
        }
    }

    fn update_viewport(&mut self) {
        self.viewport_uniform.update_viewport(&self.viewport);
        self.queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::cast_slice(&[self.viewport_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
                label: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.viewport_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
