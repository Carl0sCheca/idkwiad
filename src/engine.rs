use std::collections::HashMap;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    depth_texture: crate::texture::Texture,
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
                    features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::default(),
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

        // Depth texture
        let depth_texture =
            crate::texture::Texture::create_depth_texture(&device, &config, "depth_texture");

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
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<crate::vertex_type::DefaultVertex>()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<
                                crate::component::transform::TransformRaw,
                            >() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 5,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                    shader_location: 6,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                                    shader_location: 7,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                                    shader_location: 8,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                            ],
                        },
                    ],
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
                depth_stencil: Some(wgpu::TextureFormat::Depth32Float).map(|format| {
                    wgpu::DepthStencilState {
                        format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
        });

        let line_render_pipeline = Rc::new({
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout Descriptor"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Render Pipeline Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line.wgsl").into()),
            };

            let shader = device.create_shader_module(shader);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "v_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<crate::vertex_type::LineVertex>()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<
                                crate::component::transform::TransformRaw,
                            >() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 5,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                    shader_location: 6,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                                    shader_location: 7,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                                    shader_location: 8,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                            ],
                        },
                    ],
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
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Line,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::TextureFormat::Depth32Float).map(|format| {
                    wgpu::DepthStencilState {
                        format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
        });

        // Add pipeline to hashmap
        let mut render_pipelines = HashMap::new();
        render_pipelines.insert("Default".to_owned(), render_pipeline);
        render_pipelines.insert("Line".to_owned(), line_render_pipeline);

        // Add bindgroups to hashmap
        let mut bind_groups = HashMap::new();
        bind_groups.insert("Camera".to_owned(), Rc::new(camera_bind_group));

        // ECS
        let mut scene = hecs::World::new();

        // Spawn triangle
        let triangle_transform_1 = Arc::new(Mutex::new(
            crate::component::TransformBuild::new()
                .with_position(nalgebra_glm::vec3(0.0, 1.0, 0.0))
                // .with_rotation(nalgebra_glm::vec3(-90.0, 0.0, 180.0))
                .with_buffer(device.as_ref())
                .build(),
        ));

        scene.spawn((
            triangle_transform_1.clone(),
            crate::component::render::Render::new(
                device.as_ref(),
                (
                    vec![
                        crate::vertex_type::DefaultVertex {
                            position: [0.0, 1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [1.0, -1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [-1.0, -1.0, 0.0],
                        },
                    ],
                    vec![0u16, 2, 1],
                ),
                "Default".to_string(),
                triangle_transform_1.clone().lock().unwrap().buffer.clone(),
            ),
        ));

        let triangle_transform_2 = Arc::new(Mutex::new(
            crate::component::TransformBuild::new()
                .with_position(nalgebra_glm::vec3(3.0, 1.0, 0.0))
                .with_buffer(device.as_ref())
                .build(),
        ));

        scene.spawn((
            triangle_transform_2.clone(),
            crate::component::render::Render::new(
                device.as_ref(),
                (
                    vec![
                        crate::vertex_type::DefaultVertex {
                            position: [0.0, 1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [1.0, -1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [-1.0, -1.0, 0.0],
                        },
                    ],
                    vec![0u16, 2, 1],
                ),
                "Default".to_string(),
                triangle_transform_2.clone().lock().unwrap().buffer.clone(),
            ),
        ));

        let triangle_transform_3 = Arc::new(Mutex::new(
            crate::component::TransformBuild::new()
                .with_position(nalgebra_glm::vec3(-3.0, 1.0, 0.0))
                .with_buffer(device.as_ref())
                .build(),
        ));

        scene.spawn((
            triangle_transform_3.clone(),
            crate::component::render::Render::new(
                device.as_ref(),
                (
                    vec![
                        crate::vertex_type::DefaultVertex {
                            position: [0.0, 1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [1.0, -1.0, 0.0],
                        },
                        crate::vertex_type::DefaultVertex {
                            position: [-1.0, -1.0, 0.0],
                        },
                    ],
                    vec![0u16, 2, 1],
                ),
                "Default".to_string(),
                triangle_transform_3.clone().lock().unwrap().buffer.clone(),
            ),
        ));

        // Spawn camera
        let camera = scene.spawn((
            Arc::new(Mutex::new(
                crate::component::TransformBuild::new()
                    .with_position(nalgebra_glm::vec3(0.0, 0.0, -6.0))
                    // .with_parent(triangle_transform_1.clone())
                    .build(),
            )),
            camera,
        ));

        let lines = Arc::new(Mutex::new(
            crate::component::TransformBuild::new()
                // .with_buffer(device.as_ref())
                .build(),
        ));

        scene.spawn((
            crate::component::Render::new(
                device.as_ref(),
                crate::shapes::create_grid(100.0, 100.0, 50, 50),
                "Line".to_owned(),
                lines.clone().lock().unwrap().buffer.clone(),
            ),
            lines.clone(),
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
            depth_texture,
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.scene
            .query_mut::<&mut crate::component::render::Render>()
            .into_iter()
            .for_each(|(_id, render)| {
                render.draw(
                    &mut render_pass,
                    &self.render_pipelines[render.pipeline.as_str()],
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
                        .query_one_mut::<&crate::component::TransformType>(self.camera)
                        .unwrap()
                        .lock()
                        .unwrap();

                    ui.label(
                        egui::RichText::new(format!(
                            "position: {:.4?}\nrotation: {:.4?}",
                            transform.get_position(),
                            transform.get_rotation()
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
        self.scene
            .query_mut::<(
                &mut crate::component::TransformType,
                &mut crate::component::Render,
            )>()
            .into_iter()
            .enumerate()
            .for_each(|(i, (_, (transform, _)))| {
                let mut transform = transform.lock().unwrap();

                if i == 0 {
                    transform.add_rotation_y(0.5);
                }

                match transform.buffer.as_ref() {
                    Some(buffer) => {
                        self.queue.write_buffer(
                            buffer,
                            0,
                            bytemuck::cast_slice(&[transform.to_raw()]),
                        );
                    }
                    None => {}
                }
            });

        // Provisional camera controller
        self.scene
            .query_mut::<(
                &mut crate::component::TransformType,
                &mut crate::component::Camera,
            )>()
            .into_iter()
            .for_each(|(_, (transform, camera))| {
                let mut transform = transform.lock().unwrap();
                let position = transform.get_position();
                let right = transform.right();
                let forward = transform.forward();
                let up = transform.up();

                if self.input.0 {
                    transform.set_position(&(position + right * 0.05));
                }
                if self.input.1 {
                    transform.set_position(&(position - right * 0.05));
                }
                if self.input.2 {
                    transform.set_position(&(position + forward * 0.05));
                }
                if self.input.3 {
                    transform.set_position(&(position - forward * 0.05));
                }
                if self.input.4 {
                    transform.add_rotation_z(-0.1);
                }
                if self.input.5 {
                    transform.add_rotation_z(0.1);
                }
                if self.input.6 {
                    transform.set_position(&(position - nalgebra_glm::Vec3::y() * 0.05));
                }
                if self.input.7 {
                    transform.set_position(&(position + nalgebra_glm::Vec3::y() * 0.05));
                }

                transform.add_rotation_x(-self.mouse_delta.1 * 0.5);
                // transform.add_rotation_y(self.mouse_delta.0 * 0.5);
                transform.add_rotation_global_y(self.mouse_delta.0 * 0.5 * up.y.signum());

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
            self.depth_texture = crate::texture::Texture::create_depth_texture(
                self.device.as_ref(),
                &self.config,
                "depth_texture",
            );
            println!("New window size: {:?}", self.window.inner_size());
        }
    }
}
