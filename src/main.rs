use std::ffi::c_void;
use std::ptr;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use glow::HasContext;

type Point = [f32; 3];

fn points_to_triangles(points: &Vec<Point>) -> Vec<Point> {
    let mut triangles = Vec::with_capacity(points.len() * 6);
    for point in points {
        let a = [point[0] - 0.1, point[1] - 0.1, point[2]];
        let b = [point[0] + 0.1, point[1] - 0.1, point[2]];
        let c = [point[0] + 0.1, point[1] + 0.1, point[2]];
        let d = [point[0] - 0.1, point[1] + 0.1, point[2]];
        triangles.extend_from_slice(&[
            a, b, c,
            c, d, a,
        ]);
    }
    triangles
}

pub fn main() -> Result<()> {
    let AppCtx { gl, win, mut ev, gl_ctx: _gl_ctx } = init()?;

    let program = create_program(&gl)?;
    unsafe { gl.use_program(Some(program)) };

    let (vbo, vao) = create_vertex_buffer(&gl)?;

    unsafe { gl.clear_color(0.0, 0.0, 0.0, 1.0) };

    let frames_per_second = 60;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / frames_per_second);

    let points = vec![
        [0.0, 0.0, 0.0],
        [0.5, 0.0, 0.0],
        [0.0, 0.5, 0.0],
    ];

    'quit: loop {
        {
            use sdl3::event::Event;
            use sdl3::keyboard::Keycode;

            for event in ev.poll_iter() {
                if let Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } = event {
                    break 'quit;
                }
            }
        }

        let instant_start = Instant::now();

        let triangles = points_to_triangles(&points);
        update_vertex_buffer(&gl, &vbo, &triangles)?;

        unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };
        unsafe { gl.draw_arrays(glow::TRIANGLES, 0, triangles.len() as i32) };
        win.gl_swap_window();

        let instant_end = Instant::now();
        let duration_rendering = instant_end - instant_start;
        let duration_to_sleep = frame_duration.saturating_sub(duration_rendering);

        ::std::thread::sleep(duration_to_sleep);
    }

    unsafe { gl.delete_program(program) };
    unsafe { gl.delete_vertex_array(vao) };
    unsafe { gl.delete_buffer(vbo) };

    Ok(())
}

struct AppCtx {
    pub gl: glow::Context,
    pub gl_ctx: sdl3::video::GLContext,
    pub win: sdl3::video::Window,
    pub ev: sdl3::EventPump,
}


fn init() -> Result<AppCtx> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();

    let win = video
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .opengl()
        .build()?;

    // This needs to be created before function loading.
    // This should only be dropped after we are done with any GL.
    let gl_ctx = win.gl_create_context()?;

    let gl_loader = |s: &str| match video.gl_get_proc_address(s) {
        Some(f) => f as *const c_void,
        None => ptr::null(),
    };

    let gl = unsafe {
        glow::Context::from_loader_function(gl_loader)
    };

    let ev = sdl.event_pump()?;

    Ok(AppCtx {
        gl,
        gl_ctx,
        win,
        ev,
    })
}

const SHADER_VERTEX: &'_ str = include_str!("shader/vertex.glsl");
const SHADER_FRAGMENT: &'_ str = include_str!("shader/fragment.glsl");

fn create_program(gl: &glow::Context) -> Result<glow::NativeProgram> {
    let program = match unsafe { gl.create_program() } {
        Ok(program) => program,
        Err(e) => bail!("Could not create a program: {}", e),
    };

    let sources = [
        (glow::VERTEX_SHADER, SHADER_VERTEX),
        (glow::FRAGMENT_SHADER, SHADER_FRAGMENT),
    ];

    let mut shaders = Vec::with_capacity(sources.len());

    for (shader_type, shader_source) in sources {
        let shader = match unsafe { gl.create_shader(shader_type) } {
            Ok(shader) => shader,
            Err(e) => bail!("Could not create a shader: {}", e),
        };

        unsafe { gl.shader_source(shader, shader_source) };
        unsafe { gl.compile_shader(shader); }
        if ! unsafe { gl.get_shader_compile_status(shader) } {
            bail!("Failed to build the '{}' shader: {}", shader_type, unsafe { gl.get_shader_info_log(shader) });
        }
        unsafe { gl.attach_shader(program, shader) };

        shaders.push(shader);
    }

    unsafe { gl.link_program(program) };
    if ! unsafe { gl.get_program_link_status(program) } {
        bail!("{}", unsafe { gl.get_program_info_log(program) });
    }

    for shader in shaders {
        unsafe { gl.detach_shader(program, shader) };
        unsafe { gl.delete_shader(shader) };
    }

    Ok(program)
}

fn create_vertex_buffer(gl: &glow::Context) -> Result<(glow::NativeBuffer, glow::NativeVertexArray)> {
    let vbo = match unsafe { gl.create_buffer() } {
        Ok(buffer) => buffer,
        Err(e) => bail!("Could not create a buffer: {}", e),
    };
    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)) };

    // Describe the format of the input buffer
    let vao = match unsafe { gl.create_vertex_array() } {
        Ok(buffer) => buffer,
        Err(e) => bail!("Could not create a vertex array: {}", e),
    };
    unsafe { gl.bind_vertex_array(Some(vao)) };
    unsafe { gl.enable_vertex_attrib_array(0) };
    unsafe { gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, size_of::<Point>() as i32, 0) };

    Ok((vbo, vao))
}

fn update_vertex_buffer(gl: &glow::Context, vbo: &glow::NativeBuffer, vertices: &[[f32; 3]]) -> Result<()> {
    let vertices_u8: Vec<u8> = vertices.iter().flat_map(|v| {
        v.iter().flat_map(|c| c.to_ne_bytes())
    }).collect();

    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo)) };
    unsafe { gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8.as_slice(), glow::DYNAMIC_DRAW) };
    Ok(())
}

fn set_uniform(gl: &glow::Context, program: glow::NativeProgram, name: &str, value: f32) {
    let uniform_location = unsafe { gl.get_uniform_location(program, name) };
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    unsafe { gl.uniform_1_f32(uniform_location.as_ref(), value) }
}
