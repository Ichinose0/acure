use windows::{
    core::*, Foundation::Numerics::*, Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*, Win32::Graphics::Gdi::*,
    Win32::System::Com::*, Win32::System::LibraryLoader::*, Win32::System::Performance::*,
    Win32::System::SystemInformation::GetLocalTime, Win32::UI::Animation::*,
    Win32::UI::WindowsAndMessaging::*,
};
use std::ptr::{null, null_mut};

use crate::surface::Surface;
use crate::{AlignMode, Color, Command, LayoutMode};

#[macro_use]
macro_rules! SafeRelease {
    ($p:expr) => {
        unsafe {
            (*$p).Release();
        }
    };
}

pub struct D2D1Surface {
    hwnd: isize,
    factory: ID2D1Factory1,
    dxfactory: IDXGIFactory2,
    style: ID2D1StrokeStyle,
    manager: IUIAnimationManager,

    target: ID2D1DeviceContext,
    swapchain: IDXGISwapChain1,
    dpi: f32,
    width: u32,
    height: u32,
}

impl D2D1Surface {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Self {
        let factory = create_factory().unwrap();
        let dxfactory: IDXGIFactory2 = unsafe { CreateDXGIFactory1().unwrap() };
        let style = create_style(&factory).unwrap();
        let manager: IUIAnimationManager =
            unsafe { CoCreateInstance(&UIAnimationManager, None, CLSCTX_ALL).unwrap() };
        let transition = create_transition().unwrap();

        let mut dpi = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpi, &mut dpiy) };

        let device = create_device().unwrap();
            let target = create_render_target(&factory, &device).unwrap();
            unsafe { target.SetDpi(dpi, dpi) };

            let swapchain = create_swapchain(&device, HWND(hwnd)).unwrap();
            create_swapchain_bitmap(&swapchain, &target).unwrap();

            let brush = create_brush(&target).ok();
            let target = target;
            let swapchain = swapchain;

        Self {
            hwnd,
            factory,
            dxfactory,
            style,
            manager,
            target,
            swapchain,
            dpi,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_resize(width, height);
    }
}

impl Surface for D2D1Surface {
    fn surface_resize(&mut self, width: u32, height: u32) {
        
        // match self.render_target {
        //     Some(r) => SafeRelease!(r),
        //     None => {}
        // };

        // self.render_target = Some(render_target);
    }

    fn begin(&self) {
        unsafe { self.target.BeginDraw() };
    }

    fn end(&self) {
        unsafe {
            self.target.EndDraw(None, None).unwrap();
        }
    }

    fn command(&self, ctx: &[Command], align: AlignMode, layout: LayoutMode) {
        let mut hr = 0;

        // for c in ctx {
        //     match c {
        //         Command::FillRectangle(x, y, width, height, radius, color) => {
        //             let color = create_d3dcolorvalue(*color);
        //             let mut brush = unsafe { std::mem::zeroed() };
        //             unsafe { (*render_target).CreateSolidColorBrush(&color, null(), &mut brush) };

        //             let rect = D2D1_RECT_F {
        //                 left: *x as f32,
        //                 top: *y as f32,
        //                 right: (*x + (*width)) as f32,
        //                 bottom: (*y + (*height)) as f32,
        //             };
        //             if *radius == 0.0 {
        //                 unsafe {
        //                     (*render_target).FillRectangle(&rect, brush as *mut ID2D1Brush);
        //                 }
        //             } else {
        //                 let rounded_rect = D2D1_ROUNDED_RECT {
        //                     rect: rect,
        //                     radiusX: *radius as f32,
        //                     radiusY: *radius as f32,
        //                 };
        //                 unsafe {
        //                     (*render_target)
        //                         .FillRoundedRectangle(&rounded_rect, brush as *mut ID2D1Brush);
        //                 }
        //             }
        //             SafeRelease!(brush);
        //         }
        //         Command::WriteString(x, y, width, height, color, string) => {
        //             let color = create_d3dcolorvalue(*color);
        //             let mut string = string.encode_utf16().collect::<Vec<u16>>();
        //             string.push(0);
        //             let mut font_name = "Yu gothic".encode_utf16().collect::<Vec<u16>>();
        //             font_name.push(0);
        //             let mut lang = "en-us".encode_utf16().collect::<Vec<u16>>();
        //             lang.push(0);
        //             let mut text_format = unsafe { std::mem::zeroed() };
        //             let font_size = (*height as f32) / 2.0;
        //             unsafe {
        //                 (*self.dwrite_factory).CreateTextFormat(
        //                     font_name.as_ptr(),
        //                     null_mut(),
        //                     DWRITE_FONT_WEIGHT_REGULAR,
        //                     DWRITE_FONT_STYLE_NORMAL,
        //                     DWRITE_FONT_STRETCH_NORMAL,
        //                     font_size,
        //                     lang.as_ptr(),
        //                     &mut text_format,
        //                 );
        //                 (*text_format).SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        //                 (*text_format).SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
        //             }
        //             let mut brush = unsafe { std::mem::zeroed() };
        //             unsafe { (*render_target).CreateSolidColorBrush(&color, null(), &mut brush) };

        //             let layout_rect = D2D1_RECT_F {
        //                 left: *x as f32,
        //                 top: *y as f32,
        //                 right: (*x + (*width)) as f32,
        //                 bottom: (*y + (*height)) as f32,
        //             };

        //             unsafe {
        //                 (*render_target).DrawText(
        //                     string.as_ptr(),
        //                     string.len() as u32,
        //                     text_format,
        //                     &layout_rect,
        //                     brush as *mut ID2D1Brush,
        //                     D2D1_DRAW_TEXT_OPTIONS_CLIP,
        //                     DWRITE_MEASURING_MODE_NATURAL,
        //                 );
        //                 SafeRelease!(brush);
        //                 SafeRelease!(text_format);
        //             }
        //         }
        //     }
        // }
    }

    fn clear(&self, color: Color) {
        unsafe { self.target.Clear(Some(&D2D1_COLOR_F {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })) };
    }
}

impl Drop for D2D1Surface {
    fn drop(&mut self) {

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

fn get_dxgi_factory(device: &ID3D11Device) -> Result<IDXGIFactory2> {
    let dxdevice = device.cast::<IDXGIDevice>().unwrap();
    unsafe { dxdevice.GetAdapter().unwrap().GetParent() }
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