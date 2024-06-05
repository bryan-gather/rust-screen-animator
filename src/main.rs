extern crate gl;
extern crate glfw;

mod macros;
mod shader;
use shader::Shader;

use gl::{types::*, Enable};

// use core_foundation::base::{CFGetTypeID, CFRelease, CFTypeID, TCFType, ToVoid};
// use core_foundation::number::{CFBooleanGetValue, CFNumberGetType};
// use core_foundation_sys::number::{
//     CFBooleanGetTypeID, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
// };
// use core_foundation_sys::string::CFStringGetTypeID;
// use core_graphics::display::*;
// use core_graphics::image::CGImageRef;
// use core_graphics::window::{
//     kCGWindowListExcludeDesktopElements, kCGWindowListOptionIncludingWindow,
// };
use glfw::{Action, Context, GlfwReceiver, Key, WindowHint, WindowMode};
use image::ImageBuffer;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_void;
use std::str;
use std::{mem, ptr};
// extern crate core_foundation;
// extern crate core_graphics;

// use core_foundation::array::{CFArray, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
// use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
// use core_foundation::string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr};
// use core_graphics::display::{
//     kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CFDictionaryGetValueIfPresent,
//     CGWindowListCopyWindowInfo, CGWindowListOption,
// };

fn main() {
    // list_windows();
    // let (x, y, d, e, image) = capture_window(35974);
    let x = 1;
    let y = 1;
    let d = 4;
    let mut image = image::ImageBuffer::new(1, 1);
    let red_pixel: image::Rgba<u8> = image::Rgba([255, 255, 255, 255]);
    image.put_pixel(0, 0, red_pixel);

    let mut glfw = glfw::init_no_callbacks().unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Set window hints for transparency and no decorations
    glfw.window_hint(WindowHint::Decorated(false));
    glfw.window_hint(WindowHint::TransparentFramebuffer(true));

    let (width, height) = glfw.with_primary_monitor(|_, m| {
        let monitor = m.unwrap();
        let mode = monitor.get_video_mode().unwrap();
        (mode.width, mode.height)
    });
    // Create a full-screen window
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "Transparent Fullscreen Window",
            WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");
    window.make_current();
    window.set_key_polling(true);

    // Make the window's context current

    // Load OpenGL functions
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (ourShader, VBO, VAO, EBO, texture) = unsafe {
        // build and compile our shader program
        // ------------------------------------
        let ourShader = Shader::new(&VS_SRC, &FS_SRC);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 32] = [
            // positions       // colors        // texture coords
            1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
            0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
        ];
        let indices = [
            0, 1, 3, // first Triangle
            1, 2, 3, // second Triangle
        ];
        let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        gl::GenBuffers(1, &mut EBO);

        gl::BindVertexArray(VAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &indices[0] as *const i32 as *const c_void,
            gl::STATIC_DRAW,
        );

        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        // position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // color attribute
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        // texture coord attribute
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        // load and create a texture
        // -------------------------
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
                                                  // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // load image, create texture and generate mipmaps
        let data = image.clone().into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &data[0] as *const u8 as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        (ourShader, VBO, VAO, EBO, texture)
    };

    let start_time = glfw.get_time();

    while !window.should_close() {
        process_events(&mut window, &events);
        unsafe {
            //gl::Viewport(0, 0, 1000, 1000);
            gl::Clear(gl::COLOR_BUFFER_BIT); // Clear the screen
            gl::ClearColor(1.0, 0.0, 0.0, 0.1); // Set clear color to transparent

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let sin_x = glfw.get_time().sin() as f32;
            let cos_y = (2.5 * glfw.get_time().cos()) as f32;
            // println!("sin_x: {}", sin_x);

            let bottom = (400.0, 400.0);
            let size = (1248.0, 873.0);

            let scaledPosition = convert_to_gl_viewport(
                bottom.0 as f32,
                bottom.1 as f32,
                width as f32,
                height as f32,
            );

            let scaledSize = (2.0 * size.0 / width as f32, 2.0 * size.1 / height as f32);

            let delta = (glfw.get_time() - start_time) as f32 * 2.0;
            let delta_slow = (glfw.get_time() - start_time) as f32 * 2.0;

            let delta_squared = delta * delta;

            let offset = lerp(0.0, 0.4, delta);
            let scale_offset = lerp(0.0, 4.0, delta);

            gl::BindTexture(gl::TEXTURE_2D, texture);
            ourShader.useProgram();
            ourShader.setVec4(
                c_str!("Pos"),
                scaledPosition.0 - (offset),
                scaledPosition.1 - (offset),
                scaledSize.0 / (1.0 + scale_offset),
                scaledSize.1 / (1.0 + scale_offset),
            );

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            //Draw a simple square in the middle of the screen
        }
        render(&window);
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0) // clamp the value of
}

fn clamped_lerp(a: f32, b: f32, t: f32, min: f32, max: f32) -> f32 {
    lerp(a, b, t.clamp(0.0, 1.0)).clamp(min, max)
}

fn process_events(window: &mut glfw::Window, events: &GlfwReceiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}

// Shader sources
static VS_SRC: &'static str = "
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform vec4 Pos;

out vec3 ourColor;
out vec2 TexCoord;

void main()
{
    gl_Position = vec4(Pos.x + (aPos.x * Pos.z), Pos.y + (aPos.y * Pos.w), 0.0,  1.0);
	ourColor = aColor;
	TexCoord = vec2(aTexCoord.x, aTexCoord.y);
}
";

static FS_SRC: &'static str = "
#version 330 core
out vec4 FragColor;

in vec3 ourColor;
in vec2 TexCoord;

// texture samplers
uniform sampler2D texture1;
uniform sampler2D texture2;

void main()
{
    vec2 newTexCoord = vec2(TexCoord.x, 1.0 - TexCoord.y);
    FragColor = texture(texture1, newTexCoord);
}
";

fn render(window: &glfw::Window) {}

// fn list_windows() -> Vec<(Option<String>, u64)> {
//     let mut win_list = vec![];
//     unsafe {
//         let window_list_info = unsafe {
//             CGWindowListCopyWindowInfo(
//                 kCGWindowListOptionIncludingWindow
//                     | kCGWindowListOptionOnScreenOnly
//                     | kCGWindowListExcludeDesktopElements,
//                 kCGNullWindowID,
//             )
//         };
//         if window_list_info.is_null() {
//             println!("No windows!")
//         } else {
//             let count = unsafe { CFArrayGetCount(window_list_info) };
//             for i in 0..count {
//                 let dic_ref =
//                     unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };
//                 if dic_ref.is_null() {
//                     unsafe {
//                         CFRelease(window_list_info.cast());
//                     }
//                 }
//                 let window_owner = get_from_dict(dic_ref, "kCGWindowOwnerName");
//                 let window_id = get_from_dict(dic_ref, "kCGWindowNumber");

//                 let bounds = get_from_dict(dic_ref, "kCGWindowBounds");

//                 if let (DictEntryValue::String(name), DictEntryValue::Number(win_id)) =
//                     (window_owner, window_id)
//                 {
//                     println!("Window Name: {}, Window ID: {}", name, win_id);
//                     win_list.push((Some(name), win_id as u64));
//                 }
//             }
//         }
//     }

//     win_list
// }

// #[derive(Debug)]
// enum DictEntryValue {
//     Number(i64),
//     Bool(bool),
//     String(String),
//     Unknown,
// }
// fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
//     let key: CFString = key.into();
//     let mut value: *const c_void = std::ptr::null();
//     if unsafe { CFDictionaryGetValueIfPresent(dict, key.to_void(), &mut value) != 0 } {
//         let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
//         if type_id == unsafe { CFNumberGetTypeID() } {
//             let value = value as CFNumberRef;
//             match unsafe { CFNumberGetType(value) } {
//                 I64 => {
//                     let mut value_i64 = 0_i64;
//                     let out_value: *mut i64 = &mut value_i64;
//                     let converted = unsafe { CFNumberGetValue(value, I64, out_value.cast()) };
//                     if converted {
//                         return DictEntryValue::Number(value_i64);
//                     }
//                 }
//                 I32 => {
//                     let mut value_i32 = 0_i32;
//                     let out_value: *mut i32 = &mut value_i32;
//                     let converted = unsafe { CFNumberGetValue(value, I32, out_value.cast()) };
//                     if converted {
//                         return DictEntryValue::Number(value_i32 as i64);
//                     }
//                 }
//                 n => {
//                     eprintln!("Unsupported Number of typeId: {}", n);
//                 }
//             }
//         } else if type_id == unsafe { CFBooleanGetTypeID() } {
//             return DictEntryValue::Bool(unsafe { CFBooleanGetValue(value.cast()) });
//         } else if type_id == unsafe { CFStringGetTypeID() } {
//             let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
//             return if !c_ptr.is_null() {
//                 let c_result = unsafe { CStr::from_ptr(c_ptr) };
//                 let result = String::from(c_result.to_str().unwrap());
//                 DictEntryValue::String(result)
//             } else {
//                 // in this case there is a high chance we got a `NSString` instead of `CFString`
//                 // we have to use the objc runtime to fetch it
//                 use objc_foundation::{INSString, NSString};
//                 use objc_id::Id;
//                 let nss: Id<NSString> = unsafe { Id::from_ptr(value as *mut NSString) };
//                 let str = std::str::from_utf8(nss.deref().as_str().as_bytes());

//                 match str {
//                     Ok(s) => DictEntryValue::String(s.to_owned()),
//                     Err(_) => DictEntryValue::Unknown,
//                 }
//             };
//         } else {
//             eprintln!("Unexpected type: {}", type_id);
//         }
//     }

//     DictEntryValue::Unknown
// }

// fn capture_window(
//     window_id: u32,
// ) -> (u32, u32, u8, Vec<u8>, ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
//     let image = unsafe {
//         CGDisplay::screenshot(
//             CGRectNull,
//             kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
//             window_id as u32,
//             kCGWindowImageNominalResolution
//                 | kCGWindowImageBoundsIgnoreFraming
//                 | kCGWindowImageShouldBeOpaque,
//         )
//     }
//     .unwrap();
//     // .context(format!(
//     //     "Cannot grab screenshot from CGDisplay of window id {}",
//     //     win_id
//     // ))?;

//     let img_ref: &CGImageRef = &image;
//     // CAUTION: the width is not trust worthy, only the buffer dimensions are real
//     let (_wrong_width, h) = (img_ref.width() as u32, img_ref.height() as u32);
//     let raw_data: Vec<_> = img_ref.data().to_vec();
//     let byte_per_row = img_ref.bytes_per_row() as u32;
//     // the buffer must be as long as the row length x height
//     // ensure!(
//     //     byte_per_row * h == raw_data.len() as u32,
//     //     format!(
//     //         "Cannot grab screenshot from CGDisplay of window id {}",
//     //         win_id
//     //     )
//     // );
//     let byte_per_pixel = (img_ref.bits_per_pixel() / 8) as u8;
//     // the actual width based on the buffer dimensions
//     let w = byte_per_row / byte_per_pixel as u32;

//     println!(
//         "[WINDOW ID: {}] w: {}, h: {}, byte_per_pixel: {}, raw_data: {:?}",
//         window_id,
//         w,
//         h,
//         byte_per_pixel,
//         raw_data.len()
//     );

//     let buffer =
//         match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(w, h, raw_data.clone()) {
//             Some(buffer) => buffer,
//             None => panic!("fialed to create data"),
//         };

//     let path = format!("screenshot.png");
//     buffer.save(path).unwrap();
//     (w, h, byte_per_pixel, raw_data, buffer)
// }

// fn save_image_to_file(image: CGImage, file_path: &str) -> Result<(), String> {
//     let width = image.width() as u32;
//     let height = image.height() as u32;
//     let bits_per_component = image.bits_per_component();
//     let bytes_per_row = image.bytes_per_row();
//     let data = image.data().to_bytes();

//     let buffer = match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, data) {
//         Some(buffer) => buffer,
//         None => return Err("Failed to create image buffer".to_string()),
//     };

//     buffer.save(file_path).map_err(|e| e.to_string())
// }

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

fn convert_to_gl_viewport(x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    let x = (2.0 * x / w) - 1.0;
    let y = (2.0 * y / h) - 1.0;
    (x, y)
}
