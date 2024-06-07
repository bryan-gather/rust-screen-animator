extern crate gl;
extern crate glfw;

mod macros;
mod shader;
use cgmath::{vec3, Deg};
use shader::Shader;

use gl::{types::*, Enable};

use glfw::{Action, Context, GlfwReceiver, Key, WindowHint, WindowMode};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_void;
use std::str;
use std::{mem, ptr};

use clap::Parser;

use keyframe::{functions::EaseInOut, keyframes, mint::Point2, AnimationSequence, Keyframe};

#[derive(Parser, Debug)]
#[clap(disable_help_flag = true)]
struct Args {
    #[arg(short('i'), long("window-id"))]
    window_id: u64,

    #[arg(short('x'), long("destination-x"))]
    destination_x: f32,

    #[arg(short('y'), long("destination-y"))]
    destination_y: f32,

    #[arg(short('w'), long("destination-width"))]
    destination_width: f32,

    #[arg(short('h'), long("destination-height"))]
    destination_height: f32,
}

use std::io::Write;

mod capture;
use capture::*;

fn main() {
    let capturer = capturer::new();

    capturer.init();

    let windows = capturer.list_windows().unwrap();
    println!("Windows: {:?}", windows);

    let args: Args = Args::parse();

    let info = capturer.get_window_info(args.window_id).unwrap();

    let image = capturer.capture_window(args.window_id).unwrap();

    // list_windows();

    println!("Capture window successful!");

    let mut glfw = glfw::init_no_callbacks().unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::Floating(true));
    glfw.window_hint(glfw::WindowHint::FocusOnShow(true));

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

    let (ourShader, VBO, VAO, EBO) = unsafe {
        // build and compile our shader program
        // ------------------------------------
        let ourShader = Shader::new(&VS_SRC, &FS_SRC);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 20] = [
            // positions     colors       // texture coords
            0.5, 0.5, 0.0, 1.0, 1.0, // top right
            0.5, -0.5, 0.0, 1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0, 0.0, 0.0, // bottom left
            -0.5, 0.5, 0.0, 0.0, 1.0, // top left
        ];

        let indices: [i32; 6] = [0, 1, 3, 1, 2, 3];
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

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        // position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // texture coord attribute
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        (ourShader, VBO, VAO, EBO)
    };

    let mut back_image = ImageBuffer::new(1, 1);
    back_image.put_pixel(0, 0, image::Rgba([63, 37, 32, 255]));
    let dynamic_image = DynamicImage::ImageRgba8(back_image);

    let back_texture = create_gl_texture(&dynamic_image);
    let texture = create_gl_texture(&image);

    let mut is_complete = false;

    // TODO: Get this from monitor!
    let pixel_ratio = 2.0 as f32;
    let widthf = image.width() as f32;
    let heightf = image.height() as f32;
    let mut scale_keyframes = keyframes![
        Keyframe::new(
            keyframe::mint::Point2 {
                x: widthf,
                y: heightf
            },
            0.0,
            EaseInOut
        ),
        Keyframe::new(
            keyframe::mint::Point2 {
                x: args.destination_width,
                y: args.destination_height
            },
            1.0,
            EaseInOut
        )
    ];

    let MACOS_FUDGE_FACTOR_FOR_BAR = -25.0;
    let mut translate_keyframes = keyframes![
        Keyframe::new(
            keyframe::mint::Point2 {
                x: info.x as f32,
                y: info.y as f32 - MACOS_FUDGE_FACTOR_FOR_BAR,
            },
            0.0,
            EaseInOut
        ),
        Keyframe::new(
            keyframe::mint::Point2 {
                x: args.destination_x,
                y: args.destination_y,
            },
            1.0,
            EaseInOut
        )
    ];

    let mut rotate_keyframes = keyframes![
        Keyframe::new(0.0, 0.0, EaseInOut),
        Keyframe::new(180.0, 1.0, EaseInOut)
    ];

    while !is_complete {
        process_events(&mut window, &events);
        unsafe {
            //gl::Viewport(0, 0, 1000, 1000);
            gl::Clear(gl::COLOR_BUFFER_BIT); // Clear the screen
            gl::ClearColor(0.0, 0.0, 0.0, 0.0); // Set clear color to transparent

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let monitor_width = width as f32 * pixel_ratio;
            let monitor_height = height as f32 * pixel_ratio;
            println!("width: {}, height: {}", monitor_width, monitor_height);
            gl::Viewport(0, 0, monitor_width as i32, monitor_height as i32);

            // TODO: Actual screen resolution!
            let ortho_matrix = cgmath::ortho(0.0, monitor_width, monitor_height, 0.0, -1.0, 1.0);
            // let world_matrix = cgmath::Matrix4::<f32>::from_angle_y(cgmath::Deg::<f32>(
            //     glfw.get_time() as f32 * 0.0,
            // ));

            let time = (glfw.get_time() as f32 / 4.0).min(1.0);
            scale_keyframes.advance_to(time as f64);
            translate_keyframes.advance_to(time as f64);
            rotate_keyframes.advance_to(time as f64);

            is_complete = time > 0.99;

            let scale = scale_keyframes.now_strict().unwrap();
            let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(
                scale.x * pixel_ratio,
                scale.y * pixel_ratio,
                1.0,
            );

            let xform_matrix = cgmath::Matrix4::<f32>::from_translation(vec3(
                0.0 + (glfw.get_time() * 5.0).sin() as f32 * 0.0,
                0.0 + (glfw.get_time() * 4.0).cos() as f32 * 0.0,
                0.0,
            ));
            // let xform_matrix = cgmath::Matrix4::<f32>::from_translation(vec3(
            //     250.0 + (glfw.get_time() * 5.0).sin() as f32 * 100.0,
            //     250.0 + (glfw.get_time() * 4.0).cos() as f32 * 100.0,
            //     0.0,
            // ));

            let translate = translate_keyframes.now_strict().unwrap();

            let window_pos_matrix = cgmath::Matrix4::<f32>::from_translation(vec3(
                translate.x * pixel_ratio,
                translate.y * pixel_ratio,
                0.0,
            ));

            let offset_matrix = cgmath::Matrix4::from_translation(vec3(0.5, 0.5, 0.0));

            let rot_deg = rotate_keyframes.now_strict().unwrap();
            let rot_matrix = cgmath::Matrix4::from_angle_y(Deg(rot_deg as f32));

            let world_matrix =
                xform_matrix * window_pos_matrix * scale_matrix * offset_matrix * rot_matrix;
            // ));

            // Set the viewport to the size of the window
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, back_texture);
            ourShader.useProgram();
            // Draw a quad at 0, 0, 100, 100
            ourShader.setInt(c_str!("texture1"), 0);
            ourShader.setInt(c_str!("texture2"), 1);
            ourShader.setMat4(c_str!("projection"), &ortho_matrix);
            ourShader.setMat4(c_str!("world"), &world_matrix);

            gl::BindVertexArray(VAO);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            //Draw a simple square in the middle of the screen
        }
        render(&window);
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn create_gl_texture(image: &image::DynamicImage) -> u32 {
    // load and create a texture
    // -------------------------
    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
                                                  // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // load image, create texture and generate mipmaps
        let data = image.clone().into_rgba8().into_raw();
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
    }
    texture
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
layout (location = 1) in vec2 aTexCoord;

uniform mat4 projection;
uniform mat4 world;

out vec2 TexCoord;
out vec3 Normal;

void main()
{
    gl_Position = projection * world * vec4(aPos.x, aPos.y, aPos.z,  1.0);
	TexCoord = vec2(aTexCoord.x, aTexCoord.y);
    Normal = mat3(world) * vec3(0.0, 0.0, 1.0); // Calculate the normal vector
}
";

static FS_SRC: &'static str = "
#version 330 core
out vec4 FragColor;

in vec3 ourColor;
in vec2 TexCoord;
in vec3 Normal;

// texture samplers
uniform sampler2D texture1;
uniform sampler2D texture2;

void main()
{
    vec3 N = normalize(Normal);
    vec3 viewDir = vec3(0.0, 0.0, 1.0);
    float facing = dot(N, viewDir);
    vec4 color;
    if (facing > 0.0) {
        color = texture(texture1, TexCoord).bgra; // Front face
    } else {
        color = texture(texture2, TexCoord).bgra; // Back face
    }
    FragColor = color;
}
";

fn render(window: &glfw::Window) {}

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
