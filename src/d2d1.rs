use std::ptr::{null, null_mut};
use windows::{
    core::*, Foundation::Numerics::*, Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*, Win32::Graphics::Gdi::*,
    Win32::System::Com::*, Win32::System::LibraryLoader::*, Win32::System::Performance::*,
    Win32::System::SystemInformation::GetLocalTime, Win32::UI::Animation::*,
    Win32::UI::WindowsAndMessaging::*,
};

use crate::surface::Surface;
use crate::{AlignMode, Color, Command, LayoutMode};

impl Surface for D2D1Surface {
    fn surface_resize(&mut self, width: u32, height: u32) {
        self.resize_swapchain_bitmap().unwrap();
    }

    fn begin(&mut self) {
        if self.target.is_none() {
            let device = create_device().unwrap();
            let target = create_render_target(&self.factory, &device).unwrap();
            unsafe { target.SetDpi(self.dpi, self.dpi) };

            let swapchain = create_swapchain(&device, self.handle).unwrap();
            create_swapchain_bitmap(&swapchain, &target).unwrap();

            self.brush = create_brush(&target).ok();
            self.target = Some(target);
            self.swapchain = Some(swapchain);
            self.create_device_size_resources().unwrap();
        }

        let target = self.target.as_ref().unwrap();
        unsafe { target.BeginDraw() };
    }

    fn end(&mut self) {
        let target = self.target.as_ref().unwrap();
        unsafe {
            target.EndDraw(None, None).unwrap();
        }

        if let Err(error) = self.present(1, 0) {
            if error.code() == DXGI_STATUS_OCCLUDED {
                self.occlusion = unsafe {
                    self.dxfactory
                        .RegisterOcclusionStatusWindow(self.handle, WM_USER).unwrap()
                };
                self.visible = false;
            } else {
                self.release_device();
            }
        }
    }

    fn command(&self, ctx: &[Command], align: AlignMode, layout: LayoutMode) {
        let target = self.target.as_ref().unwrap();
        let clock = self.clock.as_ref().unwrap();
        let shadow = self.shadow.as_ref().unwrap();

        unsafe {
            self.manager.Update(get_time(self.frequency).unwrap(), None).unwrap();

            let previous = target.GetTarget().unwrap();
            target.SetTarget(clock);
            target.Clear(None);
            self.draw_clock().unwrap();
            for i in ctx {
                match i {
                    Command::FillRectangle(x,y,width,height,radius,color) => {
                        let rect = D2D_RECT_F {
                            left: *x as f32,
                            top: *y as f32,
                            right: (width+x) as f32,
                            bottom: (height+y) as f32,
                        };

                        let brush = create_brush_from_color(target,*color).unwrap();
            
                        target.FillRectangle(&rect,&brush);
                    },
                    Command::WriteString(_, _, _, _, _, _) => {},
                }
            }
            target.SetTarget(&previous);
            
            // target.DrawImage(
            //     &shadow.GetOutput().unwrap(),
            //     None,
            //     None,
            //     D2D1_INTERPOLATION_MODE_LINEAR,
            //     D2D1_COMPOSITE_MODE_SOURCE_OVER,
            // );

            target.SetTransform(&Matrix3x2::identity());

            target.DrawImage(
                clock,
                None,
                None,
                D2D1_INTERPOLATION_MODE_LINEAR,
                D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
        }
    }

    fn clear(&self, color: Color) {
        let target = self.target.as_ref().unwrap();
        let color = self.d2d1_color(color);
        unsafe { target.Clear(Some(&color)) };
    }
}

pub struct D2D1Surface {
    handle: HWND,
    factory: ID2D1Factory1,
    dxfactory: IDXGIFactory2,
    style: ID2D1StrokeStyle,
    manager: IUIAnimationManager,
    variable: IUIAnimationVariable,

    target: Option<ID2D1DeviceContext>,
    swapchain: Option<IDXGISwapChain1>,
    brush: Option<ID2D1SolidColorBrush>,
    shadow: Option<ID2D1Effect>,
    clock: Option<ID2D1Bitmap1>,
    dpi: f32,
    visible: bool,
    occlusion: u32,
    frequency: i64,
    angles: Angles,
}

#[derive(Default)]
struct Angles {
    second: f32,
    minute: f32,
    hour: f32,
}

impl Angles {
    fn now() -> Self {
        let time = unsafe { GetLocalTime() };

        let second = (time.wSecond as f32 + time.wMilliseconds as f32 / 1000.0) * 6.0;
        let minute = time.wMinute as f32 * 6.0 + second / 60.0;
        let hour = (time.wHour % 12) as f32 * 30.0 + minute / 12.0;

        Self {
            second,
            minute,
            hour,
        }
    }
}

impl D2D1Surface {
    pub fn new(hwnd: isize) -> Self {
        let factory = create_factory().unwrap();
        let dxfactory: IDXGIFactory2 = unsafe { CreateDXGIFactory1().unwrap() };
        let style = create_style(&factory).unwrap();
        let manager: IUIAnimationManager =
            unsafe { CoCreateInstance(&UIAnimationManager, None, CLSCTX_ALL).unwrap() };
        let transition = create_transition().unwrap();

        let mut dpi = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpi, &mut dpiy) };

        let mut frequency = 0;
        unsafe { QueryPerformanceFrequency(&mut frequency).unwrap() };

        let variable = unsafe {
            let variable = manager.CreateAnimationVariable(0.0).unwrap();

            manager.ScheduleTransition(&variable, &transition, get_time(frequency).unwrap()).unwrap();

            variable
        };

        Self {
            handle: HWND(hwnd),
            factory,
            dxfactory,
            style,
            manager,
            variable,
            target: None,
            swapchain: None,
            brush: None,
            shadow: None,
            clock: None,
            dpi,
            visible: false,
            occlusion: 0,
            frequency,
            angles: Angles::now(),
        }
    }

    fn d2d1_color(&self,color: Color) -> D2D1_COLOR_F {
        match color {
            Color::ARGB(a,r,g,b) => D2D1_COLOR_F { r: r as f32, g: g as f32, b: b as f32, a: a as f32}
        }
    }

    pub fn resize(&mut self) {
        self.resize_swapchain_bitmap().unwrap();
    }

    fn release_device(&mut self) {
        self.target = None;
        self.swapchain = None;
        self.release_device_resources();
    }

    fn release_device_resources(&mut self) {
        self.brush = None;
        self.clock = None;
        self.shadow = None;
    }

    fn present(&self, sync: u32, flags: u32) -> Result<()> {
        unsafe { self.swapchain.as_ref().unwrap().Present(sync, flags).ok() }
    }

    fn draw_clock(&self) -> Result<()> {
        let target = self.target.as_ref().unwrap();
        let brush = self.brush.as_ref().unwrap();

        let size = unsafe { target.GetSize() };

        #[allow(clippy::manual_clamp)]
        let radius = size.width.min(size.height).max(200.0) / 2.0 - 50.0;
        let translation = Matrix3x2::translation(size.width / 2.0, size.height / 2.0);
        unsafe { target.SetTransform(&Matrix3x2::identity()) };

        Ok(())
    }

    fn create_device_size_resources(&mut self) -> Result<()> {
        let target = self.target.as_ref().unwrap();
        let clock = self.create_clock(target)?;
        self.shadow = create_shadow(target, &clock).ok();
        self.clock = Some(clock);

        Ok(())
    }

    fn create_clock(&self, target: &ID2D1DeviceContext) -> Result<ID2D1Bitmap1> {
        let size_f = unsafe { target.GetSize() };

        let size_u = D2D_SIZE_U {
            width: (size_f.width * self.dpi / 96.0) as u32,
            height: (size_f.height * self.dpi / 96.0) as u32,
        };

        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: self.dpi,
            dpiY: self.dpi,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            ..Default::default()
        };

        unsafe { target.CreateBitmap2(size_u, None, 0, &properties) }
    }

    fn resize_swapchain_bitmap(&mut self) -> Result<()> {
        if let Some(target) = &self.target {
            let swapchain = self.swapchain.as_ref().unwrap();
            unsafe { target.SetTarget(None) };

            if unsafe {
                swapchain
                    .ResizeBuffers(0, 0, 0, DXGI_FORMAT_UNKNOWN, 0)
                    .is_ok()
            } {
                create_swapchain_bitmap(swapchain, target)?;
                self.create_device_size_resources()?;
            } else {
                self.release_device();
            }
        }

        Ok(())
    }
}

fn get_time(frequency: i64) -> Result<f64> {
    unsafe {
        let mut time = 0;
        QueryPerformanceCounter(&mut time)?;
        Ok(time as f64 / frequency as f64)
    }
}

fn create_brush(target: &ID2D1DeviceContext) -> Result<ID2D1SolidColorBrush> {
    let color = D2D1_COLOR_F {
        r: 0.92,
        g: 0.38,
        b: 0.208,
        a: 1.0,
    };

    let properties = D2D1_BRUSH_PROPERTIES {
        opacity: 0.8,
        transform: Matrix3x2::identity(),
    };

    unsafe { target.CreateSolidColorBrush(&color, Some(&properties)) }
}

fn create_brush_from_color(target: &ID2D1DeviceContext,color: Color) -> Result<ID2D1SolidColorBrush> {
    let color = match color {
        Color::ARGB(a, r, g, b) => D2D1_COLOR_F {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        },
    };

    let properties = D2D1_BRUSH_PROPERTIES {
        opacity: 1.0,
        transform: Matrix3x2::identity(),
    };

    unsafe { target.CreateSolidColorBrush(&color, Some(&properties)) }
}


fn create_shadow(target: &ID2D1DeviceContext, clock: &ID2D1Bitmap1) -> Result<ID2D1Effect> {
    unsafe {
        let shadow = target.CreateEffect(&CLSID_D2D1Shadow)?;

        shadow.SetInput(0, clock, true);
        Ok(shadow)
    }
}

fn create_factory() -> Result<ID2D1Factory1> {
    let mut options = D2D1_FACTORY_OPTIONS::default();

    if cfg!(debug_assertions) {
        options.debugLevel = D2D1_DEBUG_LEVEL_INFORMATION;
    }

    unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options)) }
}

fn create_style(factory: &ID2D1Factory1) -> Result<ID2D1StrokeStyle> {
    let props = D2D1_STROKE_STYLE_PROPERTIES {
        startCap: D2D1_CAP_STYLE_ROUND,
        endCap: D2D1_CAP_STYLE_TRIANGLE,
        ..Default::default()
    };

    unsafe { factory.CreateStrokeStyle(&props, None) }
}

fn create_transition() -> Result<IUIAnimationTransition> {
    unsafe {
        let library: IUIAnimationTransitionLibrary =
            CoCreateInstance(&UIAnimationTransitionLibrary, None, CLSCTX_ALL)?;
        library.CreateAccelerateDecelerateTransition(5.0, 1.0, 0.2, 0.8)
    }
}

fn create_device_with_type(drive_type: D3D_DRIVER_TYPE) -> Result<ID3D11Device> {
    let mut flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    if cfg!(debug_assertions) {
        flags |= D3D11_CREATE_DEVICE_DEBUG;
    }

    let mut device = None;

    unsafe {
        D3D11CreateDevice(
            None,
            drive_type,
            None,
            flags,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
        .map(|()| device.unwrap())
    }
}

fn create_device() -> Result<ID3D11Device> {
    let mut result = create_device_with_type(D3D_DRIVER_TYPE_HARDWARE);

    if let Err(err) = &result {
        if err.code() == DXGI_ERROR_UNSUPPORTED {
            result = create_device_with_type(D3D_DRIVER_TYPE_WARP);
        }
    }

    result
}

fn create_render_target(
    factory: &ID2D1Factory1,
    device: &ID3D11Device,
) -> Result<ID2D1DeviceContext> {
    unsafe {
        let d2device = factory.CreateDevice(&device.cast::<IDXGIDevice>()?)?;

        let target = d2device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

        target.SetUnitMode(D2D1_UNIT_MODE_DIPS);

        Ok(target)
    }
}

fn get_dxgi_factory(device: &ID3D11Device) -> Result<IDXGIFactory2> {
    let dxdevice = device.cast::<IDXGIDevice>()?;
    unsafe { dxdevice.GetAdapter()?.GetParent() }
}

fn create_swapchain_bitmap(swapchain: &IDXGISwapChain1, target: &ID2D1DeviceContext) -> Result<()> {
    let surface: IDXGISurface = unsafe { swapchain.GetBuffer(0)? };

    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        ..Default::default()
    };

    unsafe {
        let bitmap = target.CreateBitmapFromDxgiSurface(&surface, Some(&props))?;
        target.SetTarget(&bitmap);
    };

    Ok(())
}

fn create_swapchain(device: &ID3D11Device, window: HWND) -> Result<IDXGISwapChain1> {
    let factory = get_dxgi_factory(device)?;

    let props = DXGI_SWAP_CHAIN_DESC1 {
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        ..Default::default()
    };

    unsafe { factory.CreateSwapChainForHwnd(device, window, &props, None, None) }
}