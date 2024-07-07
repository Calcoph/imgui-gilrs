use gilrs::{Event as GilEvent, Gilrs};
use imgui::{Context, Ui};
use imgui_gilrs::GamepadHandler;
use imgui_wgpu::Renderer;
use imgui_wgpu::RendererConfig;
#[cfg(feature = "winit")]
use imgui_winit_support::WinitPlatform;
use std::thread;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Device, InstanceDescriptor, Queue, Surface,
    SurfaceConfiguration, TextureView, TextureViewDescriptor,
};
#[cfg(feature = "winit")]
use winit::{
    dpi,
    event::{Event as WinEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{self, Window},
};

const WINDOW_WIDTH: u32 = 1000;
const WINDOW_HEIGHT: u32 = 700;

#[cfg(not(feature = "winit"))]
fn main() {
    println!("To run this example:");
    println!("\tcargo run --example gamepad_tutorial --features winit")
}

#[cfg(feature = "winit")]
fn main() {
    let state = init();
    main_loop(state)
}

struct WgpuElements {
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface,
}

#[cfg(feature = "winit")]
async fn init_wgpu(winit_elem: &WinitElements) -> WgpuElements {
    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe {
        instance
            .create_surface(&winit_elem.window)
            .expect("Unable to create surface")
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Unable to request adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: wgpu::Limits {
                    max_push_constant_size: 4,
                    ..wgpu::Limits::default()
                },
                label: None,
            },
            None,
        )
        .await
        .expect("Unable to request device");

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![surface.get_capabilities(&adapter).formats[0]],
    };
    surface.configure(&device, &config);

    WgpuElements {
        device,
        queue,
        config,
        surface,
    }
}

#[cfg(feature = "winit")]
struct WinitElements {
    window: Window,
    event_loop: EventLoop<GilEvent>,
}

#[cfg(feature = "winit")]
fn init_winit() -> WinitElements {
    let event_loop = EventLoopBuilder::with_user_event().build();
    let wb = window::WindowBuilder::new()
        .with_title("imgui-gilrs-example")
        .with_inner_size(dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let window = wb.build(&event_loop).expect("Couldn't create window");

    WinitElements { window, event_loop }
}

#[cfg(feature = "winit")]
struct ImguiElements {
    context: Context,
    platform: WinitPlatform,
    renderer: Renderer,
    ui_state: UiState,
}

#[cfg(feature = "winit")]
fn init_imgui(wgpu_elem: &WgpuElements, winit_elem: &WinitElements) -> ImguiElements {
    let mut context = Context::create();
    context.io_mut().config_flags |= imgui::ConfigFlags::NAV_ENABLE_GAMEPAD;

    let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
    platform.attach_window(
        context.io_mut(),
        &winit_elem.window,
        imgui_winit_support::HiDpiMode::Default,
    );

    let renderer_config = RendererConfig {
        texture_format: wgpu_elem.config.format,
        ..Default::default()
    };

    let renderer = Renderer::new(
        &mut context,
        &wgpu_elem.device,
        &wgpu_elem.queue,
        renderer_config,
    );

    let ui_state = UiState::new();

    ImguiElements {
        context,
        platform,
        renderer,
        ui_state,
    }
}

#[cfg(feature = "winit")]
struct State {
    wgpu: WgpuElements,
    winit: WinitElements,
    imgui: ImguiElements,
    gamepad_handler: GamepadHandler,
}

#[cfg(feature = "winit")]
#[inline]
fn init() -> State {
    let winit_elem = init_winit();
    let wgpu_elem = pollster::block_on(init_wgpu(&winit_elem));
    let imgui_elem = init_imgui(&wgpu_elem, &winit_elem);
    let gamepad_handler = start_gilrs(winit_elem.event_loop.create_proxy());

    State {
        wgpu: wgpu_elem,
        winit: winit_elem,
        imgui: imgui_elem,
        gamepad_handler,
    }
}

#[cfg(feature = "winit")]
fn main_loop(state: State) {
    let State {
        wgpu:
            WgpuElements {
                device,
                queue,
                mut config,
                surface,
            },
        winit: WinitElements { window, event_loop },
        imgui:
            ImguiElements {
                mut context,
                mut platform,
                mut renderer,
                mut ui_state,
            },
        mut gamepad_handler,
    } = state;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Borrow of `event` required because we're gonna later pass it to gamepad_handler
        match &event {
            WinEvent::WindowEvent { window_id, event } if *window_id == window.id() => {
                match event {
                    WindowEvent::Resized(new_size) => {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config)
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: _,
                        new_inner_size,
                    } => {
                        config.width = new_inner_size.width;
                        config.height = new_inner_size.height;
                        surface.configure(&device, &config)
                    }
                    _ => (),
                }
            }
            WinEvent::MainEventsCleared => window.request_redraw(),
            WinEvent::RedrawRequested(window_id) if *window_id == window.id() => {
                if let Ok(texture) = surface.get_current_texture() {
                    let view = texture
                        .texture
                        .create_view(&TextureViewDescriptor::default());
                    platform
                        .prepare_frame(context.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    let ui = context.frame();

                    create_ui(ui, &mut ui_state);

                    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("ImGui Render Encoder"),
                    });
                    render(
                        &mut encoder,
                        &view,
                        &mut renderer,
                        &mut context,
                        &queue,
                        &device,
                    );

                    queue.submit(std::iter::once(encoder.finish()));
                    texture.present();
                }
            }
            #[allow(unused_variables)]
            WinEvent::UserEvent(gamepad_event) => {
                //* if not using feature `winit`, call here
                //* gamepad_handler.handle_event(context.io_mut(), &gamepad_event)
            }
            _ => (),
        }

        //* This is the equivalent to:
        //*   gamepad_handler.handle_event(io, gamepad_event)
        //* when encountering WinEvent::UserEvent(_)
        //* (See above)
        //* in that case, if you are using winit (but without the feature enabled)
        //* you'll have to call here:
        //*   platform.handle_event(context.io_mut(), &window, &event)
        gamepad_handler.handle_event(context.io_mut(), &window, &mut platform, &event)
    })
}

struct TutorialWindow {
    button_count: u32,
}

impl TutorialWindow {
    fn new() -> TutorialWindow {
        TutorialWindow { button_count: 0 }
    }
}

struct UiState {
    xbox: TutorialWindow,
    ps: TutorialWindow,
}

impl UiState {
    fn new() -> UiState {
        UiState {
            xbox: TutorialWindow::new(),
            ps: TutorialWindow::new(),
        }
    }
}

#[cfg(feature = "winit")]
fn create_ui(ui: &mut Ui, state: &mut UiState) {
    ui.window("One").build(|| {});

    ui.window("Two").build(|| {});

    ui.window("Tutorial XBOX").build(|| {
        ui.text("Hold X to show window menu");
        ui.text("While holding:");
        ui.text("\tUse RT to switch between windows");
        ui.text("\tUse the DPAD to resize the window");
        ui.text("\tUse LStick to move the window");
        ui.separator();
        ui.text("Use the DPAD to move between widgets");

        ui.separator();
        if ui.button("Press me with A") {
            state.xbox.button_count += 1
        }

        let count = state.xbox.button_count.to_string();
        ui.text(count);

        ui.separator();
        ui.button("0,0");
        ui.same_line();
        ui.button("1,0");
        ui.same_line();
        ui.button("2,0");
        ui.button("0,1");
        ui.same_line();
        ui.button("1,1");
        ui.same_line();
        ui.button("2,1");
        ui.button("0,2");
        ui.same_line();
        ui.button("1,2");
        ui.same_line();
        ui.button("2,2");
    });

    ui.window("Tutorial PlayStation").build(|| {
        ui.text("Hold Square to show window menu");
        ui.text("While holding:");
        ui.text("\tUse R2 to switch between windows");
        ui.text("\tUse the DPAD to resize the window");
        ui.text("\tUse LStick to move the window");
        ui.separator();
        ui.text("Use the DPAD to move between widgets");

        ui.separator();
        if ui.button("Press me with X") {
            state.ps.button_count += 1
        }

        let count = state.ps.button_count.to_string();
        ui.text(count);

        ui.separator();
        ui.button("0,0");
        ui.same_line();
        ui.button("1,0");
        ui.same_line();
        ui.button("2,0");
        ui.button("0,1");
        ui.same_line();
        ui.button("1,1");
        ui.same_line();
        ui.button("2,1");
        ui.button("0,2");
        ui.same_line();
        ui.button("1,2");
        ui.same_line();
        ui.button("2,2");
    });
}

#[cfg(feature = "winit")]
fn render(
    encoder: &mut CommandEncoder,
    view: &TextureView,
    renderer: &mut Renderer,
    context: &mut Context,
    queue: &Queue,
    device: &Device,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    renderer
        .render(context.render(), queue, device, &mut render_pass)
        .expect("Rendering failed");
}

#[cfg(feature = "winit")]
fn start_gilrs(event_loop: EventLoopProxy<GilEvent>) -> GamepadHandler {
    thread::spawn(move || {
        let mut listener = Gilrs::new().expect("Couldn't initialize gilrs");
        loop {
            while let Some(ev) = listener.next_event() {
                event_loop
                    .send_event(ev)
                    .expect("Event loop no longer exists");
            }
        }
    });

    GamepadHandler::new()
}
