use std::collections::HashMap;
use std::rc::Rc;

pub struct Engine {
    window: Rc<winit::window::Window>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scene: hecs::World,
    render_pipelines: HashMap<String, Rc<wgpu::RenderPipeline>>,
    egui: (
        egui_wgpu_backend::RenderPass,
        egui_winit::State,
        egui::Context,
    ),
}

impl Engine {
    pub async fn new(window: Rc<winit::window::Window>) -> Self {
        // Surface, device, queue and config
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window.as_ref()) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.as_ref().inner_size().width,
            height: window.as_ref().inner_size().height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        surface.configure(&device, &config);

        // ECS
        let mut scene = hecs::World::new();

        // Pipelines
        let render_pipeline = Rc::new({
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout Descriptor"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Render Pipeline Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/default.wgsl").into()),
            };

            let shader = device.create_shader_module(shader);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "v_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<crate::vertex_type::DefaultVertex>()
                            as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "f_main",
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
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
        });

        let mut render_pipelines = HashMap::new();
        render_pipelines.insert("Default".to_owned(), render_pipeline);

        // Vertex, index buffer

        let vertex = vec![
            crate::vertex_type::DefaultVertex {
                position: [0.0, 1.0, 1.0],
            },
            crate::vertex_type::DefaultVertex {
                position: [1.0, -1.0, 1.0],
            },
            crate::vertex_type::DefaultVertex {
                position: [-1.0, -1.0, 1.0],
            },
        ];

        let indices = vec![0u16, 2, 1];

        let vertex_buffer = std::sync::Arc::new(wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer Init"),
                contents: bytemuck::cast_slice(&vertex),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        let index_buffer = std::sync::Arc::new(wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer Init"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));

        scene.spawn((
            crate::component::transform::Transform::default(),
            crate::component::render::Render {
                vertex_buffer: vertex_buffer.clone(),
                index_buffer: index_buffer.clone(),
                pipeline: "Default",
                index_count: 3,
            },
        ));

        Self {
            surface,
            config,
            scene,
            render_pipelines,
            egui: (
                egui_wgpu_backend::RenderPass::new(&device, surface_format, 1),
                egui_winit::State::new(4098, window.as_ref()),
                egui::Context::default(),
            ),
            device,
            queue,
            window,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output_frame = self.surface.get_current_texture()?;
        let view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.scene
            .query_mut::<&mut crate::component::render::Render>()
            .into_iter()
            .for_each(|(_id, render)| {
                render.draw(
                    &mut render_pass,
                    &self.render_pipelines[render.pipeline],
                    None,
                )
            });

        drop(render_pass);

        // EGUI
        let input = self.egui.1.take_egui_input(self.window.as_ref());
        let output = self.egui.2.run(input, |ctx| {
            egui::Area::new("debug_info")
                .fixed_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(format!("test"))
                            .background_color(egui::Color32::BLACK)
                            .color(egui::Color32::WHITE)
                            .size(20.0),
                    );
                });
        });

        let paint_jobs = self.egui.2.tessellate(output.shapes);

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: self.window.scale_factor() as f32,
        };

        self.egui
            .0
            .add_textures(&self.device, &self.queue, &output.textures_delta)
            .unwrap();

        self.egui.0.remove_textures(output.textures_delta).unwrap();

        self.egui
            .0
            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        self.egui
            .0
            .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        self.queue.submit(std::iter::once(encoder.finish()));
        output_frame.present();

        Ok(())
    }

    pub fn update(&mut self) {}

    pub fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            println!("new window size: {:?}", self.window.inner_size());
        }
    }
}
