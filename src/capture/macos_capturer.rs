use core_foundation::base::{CFGetTypeID, CFRelease, CFTypeID, TCFType, ToVoid};
use core_foundation::dictionary::CFDictionaryGetTypeID;
use core_foundation::number::{CFBooleanGetValue, CFNumberGetType};
use core_foundation::string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr};
use core_foundation_sys::number::{
    CFBooleanGetTypeID, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
};
use core_foundation_sys::string::CFStringGetTypeID;
use core_graphics::image::CGImageRef;
use core_graphics::window::{
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionIncludingWindow,
};
use core_graphics::{display::*, window};
use image::ImageBuffer;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_void;

use crate::capturer::Capturer;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGRectMakeWithDictionaryRepresentation(
        dict: CFDictionaryRef,
        rect: *mut CGRect,
    ) -> boolean_t;
}

pub(crate) struct MacOsCapturer;

impl Capturer for MacOsCapturer {
    fn init(&self) {
        // No-op on Mac
    }

    fn list_windows(&self) -> Result<Vec<(u64, String)>, ()> {
        println!("trait list_windows");
        let window_list = list_windows();

        let refined = window_list
            .into_iter()
            .map(|(name, id, _x, _y)| (id, name.unwrap_or("".to_string())))
            .collect();
        Ok(refined)
    }

    fn get_window_info(&self, window_id: u64) -> Option<crate::capturer::WindowInfo> {
        let window = list_windows()
            .into_iter()
            .find(|(name, id, _x, _y)| *id == window_id)
            .map(|(name, id, x, y)| crate::capturer::WindowInfo {
                name: name.unwrap_or("".to_string()),
                x,
                y,
            });
        window
    }

    fn capture_window(&self, window_id: u64) -> Result<image::DynamicImage, ()> {
        let (x, y, d, e, image) = capture_window(window_id as u32);
        Ok(image.into())
    }
}

fn list_windows() -> Vec<(Option<String>, u64, f32, f32)> {
    let mut win_list = vec![];
    println!("Listing windows");
    unsafe {
        let window_list_info = unsafe {
            CGWindowListCopyWindowInfo(
                kCGWindowListOptionIncludingWindow
                    | kCGWindowListOptionOnScreenOnly
                    | kCGWindowListExcludeDesktopElements,
                kCGNullWindowID,
            )
        };
        if window_list_info.is_null() {
            println!("No windows!")
        } else {
            let count = unsafe { CFArrayGetCount(window_list_info) };
            println!("Some windows: {}", count);
            for i in 0..count {
                let dic_ref =
                    unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };
                if dic_ref.is_null() {
                    unsafe {
                        CFRelease(window_list_info.cast());
                    }
                }
                let window_owner = get_from_dict(dic_ref, "kCGWindowOwnerName");
                let window_id = get_from_dict(dic_ref, "kCGWindowNumber");

                let bounds = get_from_dict(dic_ref, "kCGWindowBounds");

                if let (DictEntryValue::String(name), DictEntryValue::Number(win_id)) =
                    (window_owner, window_id)
                {
                    if let DictEntryValue::Rectangle(rect) = bounds {
                        println!(
                            "Window Name: {}, Window ID: {} Bounds: {:?}",
                            name, win_id, rect
                        );
                        win_list.push((
                            Some(name),
                            win_id as u64,
                            rect.origin.x as f32,
                            rect.origin.y as f32,
                        ));
                    }
                }
            }
        }
    }

    win_list
}

#[derive(Debug)]
enum DictEntryValue {
    Number(i64),
    Bool(bool),
    String(String),
    Rectangle(CGRect),
    Unknown,
}
fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;
            match unsafe { CFNumberGetType(value) } {
                I64 => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted = unsafe { CFNumberGetValue(value, I64, out_value.cast()) };
                    if converted {
                        return DictEntryValue::Number(value_i64);
                    }
                }
                I32 => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted = unsafe { CFNumberGetValue(value, I32, out_value.cast()) };
                    if converted {
                        return DictEntryValue::Number(value_i32 as i64);
                    }
                }
                n => {
                    eprintln!("Unsupported Number of typeId: {}", n);
                }
            }
        } else if type_id == unsafe { CFDictionaryGetTypeID() } && key == "kCGWindowBounds" {
            let rect: CGRect = unsafe {
                let mut rect = std::mem::zeroed();
                CGRectMakeWithDictionaryRepresentation(value.cast(), &mut rect);
                rect
            };

            return DictEntryValue::Rectangle(rect);
        } else if type_id == unsafe { CFBooleanGetTypeID() } {
            return DictEntryValue::Bool(unsafe { CFBooleanGetValue(value.cast()) });
        } else if type_id == unsafe { CFStringGetTypeID() } {
            let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
            return if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                DictEntryValue::String(result)
            } else {
                // in this case there is a high chance we got a `NSString` instead of `CFString`
                // we have to use the objc runtime to fetch it
                use objc_foundation::{INSString, NSString};
                use objc_id::Id;
                let nss: Id<NSString> = unsafe { Id::from_ptr(value as *mut NSString) };
                let str = std::str::from_utf8(nss.deref().as_str().as_bytes());

                match str {
                    Ok(s) => DictEntryValue::String(s.to_owned()),
                    Err(_) => DictEntryValue::Unknown,
                }
            };
        } else {
            eprintln!("Unexpected type: {}", type_id);
        }
    }

    DictEntryValue::Unknown
}

fn capture_window(
    window_id: u32,
) -> (u32, u32, u8, Vec<u8>, ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let image = unsafe {
        CGDisplay::screenshot(
            CGRectNull,
            kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
            window_id as u32,
            kCGWindowImageNominalResolution
                | kCGWindowImageBoundsIgnoreFraming
                | kCGWindowImageShouldBeOpaque,
        )
    }
    .unwrap();
    // .context(format!(
    //     "Cannot grab screenshot from CGDisplay of window id {}",
    //     win_id
    // ))?;

    let img_ref: &CGImageRef = &image;
    // CAUTION: the width is not trust worthy, only the buffer dimensions are real
    let (_wrong_width, h) = (img_ref.width() as u32, img_ref.height() as u32);
    let raw_data: Vec<_> = img_ref.data().to_vec();
    let byte_per_row = img_ref.bytes_per_row() as u32;
    // the buffer must be as long as the row length x height
    // ensure!(
    //     byte_per_row * h == raw_data.len() as u32,
    //     format!(
    //         "Cannot grab screenshot from CGDisplay of window id {}",
    //         win_id
    //     )
    // );
    let byte_per_pixel = (img_ref.bits_per_pixel() / 8) as u8;
    // the actual width based on the buffer dimensions
    let w = byte_per_row / byte_per_pixel as u32;

    // Use CoreGraphics library to get window info
    let window_info_list = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
            window_id,
        )
    };

    println!(
        "[WINDOW ID: {}] w: {}, h: {}, byte_per_pixel: {}, raw_data: {:?}",
        window_id,
        w,
        h,
        byte_per_pixel,
        raw_data.len()
    );

    let buffer =
        match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(w, h, raw_data.clone()) {
            Some(buffer) => buffer,
            None => panic!("fialed to create data"),
        };

    let path = format!("screenshot.png");
    buffer.save(path).unwrap();
    (w, h, byte_per_pixel, raw_data, buffer)
}
