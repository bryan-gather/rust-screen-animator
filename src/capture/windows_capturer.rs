///
/// Windows capturer implementation.
///
/// Currently uses DXGI for capturing, but can consider migrating to Windows.Graphics>Capture API
///
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use std::sync::mpsc::channel;

use windows::core::{ComInterface, IInspectable, Interface, Result, HSTRING};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat};
use windows::Storage::{CreationCollisionOption, FileAccessMode, StorageFolder};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::{
    Graphics::Capture::IGraphicsCaptureItemInterop, RoInitialize, RO_INIT_MULTITHREADED,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetDesktopWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible,
};

use crate::capturer::Capturer;

pub(crate) struct WindowsCapturer {
    // ...
}

impl Capturer for WindowsCapturer {
    // ...

    fn init(&self) {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED).unwrap();
        }
    }

    fn capture_window(&self, window_id: u64) -> core::result::Result<image::DynamicImage, ()> {
        let res = capture_window(HWND(window_id as isize));
        // TODO
        res.map_err(|_| ())
    }

    fn list_windows(&self) -> core::result::Result<Vec<(u64, std::ffi::OsString)>, ()> {
        unsafe {
            EnumWindows(Some(enum_windows_proc), LPARAM(0));
        }
        Ok([].to_vec())
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut buf = vec![0; (len + 1) as usize];
    if IsWindowVisible(hwnd) == TRUE {
        let length = GetWindowTextW(hwnd, &mut buf);
        if length > 0 {
            let window_title = OsString::from_wide(&buf[..length as usize]);
            println!("Window handle: {:?}, Title: {:?}", hwnd, window_title);
        }
    }
    TRUE
}
pub fn get_d3d_interface_from_object<S: Interface + ComInterface, R: Interface + ComInterface>(
    object: &S,
) -> Result<R> {
    let access: IDirect3DDxgiInterfaceAccess = object.cast()?;
    let object = unsafe { access.GetInterface::<R>()? };
    Ok(object)
}

fn create_capture_item_for_window(window_handle: HWND) -> Result<GraphicsCaptureItem> {
    let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { interop.CreateForWindow(window_handle) }
}
fn capture_window(hwnd: HWND) -> Result<image::DynamicImage> {
    unsafe {
        let item = create_capture_item_for_window(hwnd)?;
        let size = item.Size()?;
        let width = size.Width as u32;
        let height = size.Height as u32;

        let mut device = None;

        let device = windows::Win32::Graphics::Direct3D11::D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
        .map(|()| device.unwrap())?;

        let context: ID3D11DeviceContext = device.GetImmediateContext()?;
        let dxgi_device: IDXGIDevice = device.cast()?;
        let d3d_device: IDirect3DDevice =
            unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? }.cast()?;
        let capturer = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &d3d_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            size,
        )?;

        let session = capturer.CreateCaptureSession(&item)?;
        let (sender, receiver) = channel();
        capturer.FrameArrived(
            &TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new({
                move |frame_pool, _| {
                    let frame_pool = frame_pool.as_ref().unwrap();
                    let frame = frame_pool.TryGetNextFrame()?;
                    sender.send(frame).unwrap();
                    Ok(())
                }
            }),
        )?;

        session.StartCapture()?;
        let texture = unsafe {
            let frame = receiver.recv().unwrap();

            let source_texture: ID3D11Texture2D = get_d3d_interface_from_object(&frame.Surface()?)?;
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            source_texture.GetDesc(&mut desc);
            desc.BindFlags = 0;
            desc.MiscFlags = 0;
            desc.Usage = D3D11_USAGE_STAGING;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            let copy_texture = {
                let mut texture = None;
                device.CreateTexture2D(&desc, None, Some(&mut texture))?;
                texture.unwrap()
            };

            context.CopyResource(Some(&copy_texture.cast()?), Some(&source_texture.cast()?));

            session.Close()?;
            capturer.Close()?;

            copy_texture
        };
        let bits = unsafe {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut desc as *mut _);

            let resource: ID3D11Resource = texture.cast()?;
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            context.Map(
                Some(&resource.clone()),
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            )?;

            // Get a slice of bytes
            let slice: &[u8] = {
                std::slice::from_raw_parts(
                    mapped.pData as *const _,
                    (desc.Height * mapped.RowPitch) as usize,
                )
            };

            let bytes_per_pixel = 4;
            let mut bits = vec![0u8; (desc.Width * desc.Height * bytes_per_pixel) as usize];
            for row in 0..desc.Height {
                let data_begin = (row * (desc.Width * bytes_per_pixel)) as usize;
                let data_end = ((row + 1) * (desc.Width * bytes_per_pixel)) as usize;
                let slice_begin = (row * mapped.RowPitch) as usize;
                let slice_end = slice_begin + (desc.Width * bytes_per_pixel) as usize;
                bits[data_begin..data_end].copy_from_slice(&slice[slice_begin..slice_end]);
            }

            context.Unmap(Some(&resource), 0);

            bits
        };

        let buffer = image::ImageBuffer::from_raw(width, height, bits).unwrap();
        Ok(image::DynamicImage::ImageRgba8(buffer))
    }
}
