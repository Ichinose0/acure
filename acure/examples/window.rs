use acure::{gdi::GDISurface, Acure, Color, Command};
use raw_window_handle::{HasRawWindowHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

    let acure = Acure::new();
    let surface;
    let handle = window.window_handle().unwrap().as_raw();
    match handle {
        raw_window_handle::RawWindowHandle::UiKit(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::AppKit(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Orbital(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Xlib(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Xcb(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Wayland(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Drm(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Gbm(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            surface = GDISurface::new(isize::from(handle.hwnd));
        }
        raw_window_handle::RawWindowHandle::WinRt(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Web(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::WebCanvas(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::WebOffscreenCanvas(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::AndroidNdk(_) => {
            panic!("This sample is available only Windows")
        }
        raw_window_handle::RawWindowHandle::Haiku(_) => {
            panic!("This sample is available only Windows")
        }
        _ => panic!("This sample is available only Windows"),
    }

    acure.push(Command::Clear(Color::ARGB(255, 100, 50, 0)));
    acure.push(Command::WriteString(10,10,50,50,Color::ARGB(255,0,100,0),String::from("Hello")));

    event_loop.run(move |event, elwt| {
        println!("{event:?}");

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    acure.write(&surface);
                    window.pre_present_notify();
                }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();
            }

            _ => (),
        }
    })
}
