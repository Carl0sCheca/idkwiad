use std::collections::HashMap;
use std::{rc::Rc, sync::Arc};

pub struct Engine {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scene: hecs::World,
    render_pipelines: HashMap<String, Rc<wgpu::RenderPipeline>>,
    bind_groups: HashMap<String, Rc<wgpu::BindGroup>>,
    egui: (
        egui_wgpu_backend::RenderPass,
        egui_winit::State,
        egui::Context,
    ),
    input: (bool, bool, bool, bool, bool, bool, bool, bool),
    pub mouse_delta: (f32, f32),
    camera: hecs::Entity,
}

impl Engine {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
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

        let device = Arc::new(device);

        let surface_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.as_ref().inner_size().width,
            height: window.as_ref().inner_size().height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        surface.configure(&device, &config);

        // Camera
        let (camera, camera_bind_group, camera_bind_group_layout) = crate::component::Camera::new(
            crate::component::CameraType::Perspective,
            window.clone(),
            device.clone(),
        );

        // Pipeline
        let render_pipeline = Rc::new({
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout Descriptor"),
                    bind_group_layouts: &[&camera_bind_group_layout],
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

        // Add pipeline to hashmap
        let mut render_pipelines = HashMap::new();
        render_pipelines.insert("Default".to_owned(), render_pipeline);

        let mut bind_groups = HashMap::new();
        bind_groups.insert("Camera".to_owned(), Rc::new(camera_bind_group));

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

        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device.as_ref(),
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer Init"),
                contents: bytemuck::cast_slice(&vertex),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device.as_ref(),
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer Init"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        // ECS
        let mut scene = hecs::World::new();

        // Spawn triangle
        scene.spawn((
            crate::component::transform::Transform::default(),
            crate::component::render::Render {
                vertex_buffer,
                index_buffer,
                pipeline: "Default",
                index_count: 3,
            },
        ));

        // Spawn camera
        let camera = scene.spawn((
            crate::component::TransformBuild::new()
                .with_position(nalgebra_glm::vec3(0.0, 0.0, -2.0))
                .with_rotation(nalgebra_glm::zero())
                .build(),
            camera,
        ));

        Self {
            surface,
            config,
            scene,
            render_pipelines,
            egui: (
                egui_wgpu_backend::RenderPass::new(&device, surface_format, 1),
                egui_winit::State::new(2048, window.as_ref()),
                egui::Context::default(),
            ),
            device,
            queue,
            window,
            input: (false, false, false, false, false, false, false, false),
            bind_groups,
            mouse_delta: (0.0, 0.0),
            camera,
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
                    Some(vec![(0, &self.bind_groups["Camera"])]),
                )
            });

        drop(render_pass);

        // EGUI
        let input = self.egui.1.take_egui_input(self.window.as_ref());
        let output = self.egui.2.run(input, |ctx| {
            egui::Area::new("debug_info")
                .fixed_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    let transform = self
                        .scene
                        .query_one_mut::<&crate::component::Transform>(self.camera)
                        .unwrap();

                    ui.label(
                        egui::RichText::new(format!(
                            "position: {:.4?}\nrotation: {:.4?}",
                            transform.position, transform.rotation
                        ))
                        .background_color(egui::Color32::from_rgba_premultiplied(0, 0, 0, 160))
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

    pub fn update(&mut self) {
        // Provisional camera controller
        self.scene
            .query_mut::<(
                &mut crate::component::Transform,
                &mut crate::component::Camera,
            )>()
            .into_iter()
            .for_each(|(_, (transform, camera))| {
                if self.input.0 {
                    transform.position += transform.right() * 0.01;
                }
                if self.input.1 {
                    transform.position -= transform.right() * 0.01;
                }
                if self.input.2 {
                    transform.position += transform.forward() * 0.01;
                }
                if self.input.3 {
                    transform.position -= transform.forward() * 0.01;
                }
                if self.input.4 {
                    transform.add_rotation_z(-0.5);
                }
                if self.input.5 {
                    transform.add_rotation_z(0.5);
                }
                if self.input.6 {
                    transform.position.y -= 0.01;
                }
                if self.input.7 {
                    transform.position.y += 0.01;
                }

                transform.add_rotation_y(self.mouse_delta.0 * 0.5);
                transform.add_rotation_x(-self.mouse_delta.1 * 0.5);

                self.window
                    .set_cursor_position(winit::dpi::PhysicalPosition { x: 640.0, y: 360.0 })
                    .unwrap();

                self.mouse_delta = (0.0, 0.0);

                camera.update(&transform, &self.queue);
            });
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        fn is_pressed(s: &winit::event::ElementState) -> bool {
            match s {
                winit::event::ElementState::Pressed => true,
                _ => false,
            }
        }

        match event {
            winit::event::WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match key {
                winit::event::VirtualKeyCode::A => {
                    self.input.0 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::D => {
                    self.input.1 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::W => {
                    self.input.2 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::S => {
                    self.input.3 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::Q => {
                    self.input.4 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::E => {
                    self.input.5 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::LControl => {
                    self.input.6 = is_pressed(state);
                    true
                }
                winit::event::VirtualKeyCode::Space => {
                    self.input.7 = is_pressed(state);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            println!("New window size: {:?}", self.window.inner_size());
        }
    }
}
