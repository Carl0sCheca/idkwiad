mod component;
mod engine;
mod vertex_type;

pub async fn run() {
    // Logs
    env_logger::init_from_env(
        env_logger::Env::default()
            .filter_or("MY_LOG_LEVEL", "warn")
            .write_style_or("MY_LOG_STYLE", "always"),
    );

    let event_loop = winit::event_loop::EventLoop::new();
    let window = std::sync::Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("aaaaaaaa")
            .with_decorations(true)
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            // .with_position(winit::dpi::PhysicalPosition::new(0.0, 0.0))
            .build(&event_loop)
            .unwrap(),
    );

    let mut engine = crate::engine::Engine::new(window.clone()).await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !engine.input(event) {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    _ => {}
                }
            }
            match event {
                winit::event::WindowEvent::Resized(new_size) => engine.resize(*new_size),
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    engine.resize(**new_inner_size)
                }
                _ => {}
            }
        }
        winit::event::Event::DeviceEvent {
            event: winit::event::DeviceEvent::MouseMotion { delta },
            ..
        } => engine.mouse_delta = (delta.0 as f32, delta.1 as f32),
        winit::event::Event::RedrawRequested(window_id) if window_id == window.id() => {
            engine.update();

            match engine.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => engine.resize(window.as_ref().inner_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                _ => {}
            }
        }
        winit::event::Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
