use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    keyboard::{Key, NamedKey},
};
use wgpu::util::DeviceExt;
use libsumi::{Color, Camera, Vec3, Mat4}; 

// --- 1. Enhanced Uniforms (Mouse + Camera) ---
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view: [[f32; 4]; 4],       // Camera View Matrix
    proj: [[f32; 4]; 4],       // Camera Projection Matrix
    resolution: [f32; 2],      // Screen Width, Height
    time: f32,                 // Seconds since start
    padding: f32,              // Alignment padding
    mouse: [f32; 4],           // xy = current pos, zw = click pos
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    
    let window = Arc::new(WindowBuilder::new()
        .with_title("GYOSHO: Hanga Runtime (Mouse & Camera Enabled)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap());

    let mut state = pollster::block_on(State::new(window.clone()));

    println!("--- Hanga Initialized ---");
    println!("GPU: {}", state.adapter.get_info().name);
    println!("Controls: Mouse to interact, Scroll to Zoom, ESC to Quit");

    let start_time = std::time::Instant::now();

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == state.window.id() => {
                // Input returns true if it consumed the event (mouse/keyboard)
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested |
                        WindowEvent::KeyboardInput {
                            event: KeyEvent { state: ElementState::Pressed, logical_key: Key::Named(NamedKey::Escape), .. },
                            ..
                        } => target.exit(),
                        
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        
                        WindowEvent::RedrawRequested => {
                            let time = start_time.elapsed().as_secs_f32();
                            state.update(time);
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => state.window.request_redraw(),
            _ => {}
        }
    }).unwrap();
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    adapter: wgpu::Adapter,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    
    // --- State Variables ---
    camera: Camera,
    mouse_pos: [f32; 2],
    mouse_click: [f32; 2],
    mouse_pressed: bool,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // --- Camera Setup ---
        let camera = Camera::default(); // Uses libsumi default (0,0,5)

        // --- Uniform Setup ---
        let uniforms = Uniforms {
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            resolution: [size.width as f32, size.height as f32],
            time: 0.0,
            padding: 0.0,
            mouse: [0.0; 4],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, // Available in both now
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
            label: Some("uniform_bind_group"),
        });

        // --- Shader Loading ---
        let header_source = include_str!("header.wgsl");
        let stdlib_source = include_str!("stdlib.wgsl");
        let lib_source = include_str!("generated.wgsl");
        let driver_source = include_str!("shader.wgsl");
        let final_source = format!("{}\n{}\n{}\n{}", header_source, stdlib_source, lib_source, driver_source);
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sumi Shader Module"),
            source: wgpu::ShaderSource::Wgsl(final_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader_module, entry_point: "vs_main", buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface, device, queue, config, size, window, adapter,
            render_pipeline, uniform_buffer, uniform_bind_group,
            camera,
            mouse_pos: [0.0, 0.0],
            mouse_click: [0.0, 0.0],
            mouse_pressed: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Mouse Move
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = [position.x as f32, position.y as f32];
                if self.mouse_pressed {
                    self.mouse_click = self.mouse_pos;
                }
                true
            }
            // Mouse Click
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                if self.mouse_pressed {
                    self.mouse_click = self.mouse_pos;
                } else {
                    // On release, Shadertoy conventions say zw should be negative or capture last click
                    // We'll keep it simple for now.
                    self.mouse_click = [0.0, 0.0]; 
                }
                true
            }
            // Zoom (Scroll)
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                // Move camera closer/further
                self.camera.position.z -= scroll; 
                if self.camera.position.z < 0.1 { self.camera.position.z = 0.1; }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, time: f32) {
        // Update Camera Matrices
        let view = self.camera.get_view_matrix();
        let proj = self.camera.get_projection_matrix();

        let uniforms = Uniforms {
            view: view.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            resolution: [self.size.width as f32, self.size.height as f32],
            time,
            padding: 0.0,
            // iMouse: xy = current, zw = click/drag
            mouse: [
                self.mouse_pos[0], 
                self.size.height as f32 - self.mouse_pos[1], // Flip Y for Shadertoy convention (Bottom-Left is 0,0)
                self.mouse_click[0], 
                if self.mouse_pressed { self.size.height as f32 - self.mouse_click[1] } else { -1.0 }
            ],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: Color::SUMI_INK.r as f64,
                            g: Color::SUMI_INK.g as f64,
                            b: Color::SUMI_INK.b as f64,
                            a: Color::SUMI_INK.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1); 
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
