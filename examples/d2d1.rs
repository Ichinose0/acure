use acure::d2d1::D2D1Surface;
use acure::{Acure, AlignMode, Color, Command, LayoutMode};
use raw_window_handle::HasWindowHandle;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let mut acure = Acure::new();
    let mut surface;
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
            let size = window.inner_size();
            surface = D2D1Surface::new(isize::from(handle.hwnd),size.width,size.height);
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

    acure.set_layout_mode(LayoutMode::AdjustSize);
    acure.set_align_mode(AlignMode::CenterAligned);

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::Resized(size) => {
                surface.resize(size.width,size.height);
            }
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::RedrawRequested => {
                acure.push(Command::FillRectangle(
                    10,
                    10,
                    240,
                    40,
                    10.0,
                    Color::ARGB(255, 255, 0, 255),
                ));
                acure.push(Command::WriteString(
                    10,
                    10,
                    240,
                    40,
                    Color::ARGB(255, 255, 255, 255),
                    String::from("Direct2D"),
                ));
                acure.write(&mut surface);
                acure.clear();
                window.pre_present_notify();
            }
            _ => (),
        },
        Event::AboutToWait => {
            window.request_redraw();
        }

        _ => (),
    })
}
