use core::slice;
use std::ffi::c_void;
use std::ptr;
use std::time::{Duration, Instant};

use anyhow::{bail, ensure, Context, Result};
use cgmath::{Point2, Point3};
use glow::HasContext;

type Position = Point3<f32>;
type TexCoord = Point2<f32>;

pub fn main() -> Result<()> {
    let AppCtx { gl, win, mut ev, gl_ctx: _gl_ctx } = init()?;

    let program = create_program(&gl)?;
    unsafe { gl.use_program(Some(program)) };

    let buffers = create_buffers(&gl, program)?;

    unsafe { gl.clear_color(0.0, 0.0, 0.0, 1.0) };

    let frames_per_second = 60;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / frames_per_second);

    let points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.0, 0.0),
        Point3::new(0.0, 0.5, 0.0),
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

        let vertices = update_buffers(&gl, &buffers, &points)?;

        unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };
        unsafe { gl.draw_arrays(glow::TRIANGLES, 0, vertices) };
        win.gl_swap_window();

        let instant_end = Instant::now();
        let duration_rendering = instant_end - instant_start;
        let duration_to_sleep = frame_duration.saturating_sub(duration_rendering);

        ::std::thread::sleep(duration_to_sleep);
    }

    unsafe { gl.delete_program(program) };
    delete_buffers(&gl, buffers);

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

        let success = unsafe { gl.get_shader_compile_status(shader) };
        if !success || cfg!(debug_assertions) {
            let log = unsafe { gl.get_shader_info_log(shader) };
            println!("Shader '{}' info log:\n{}", shader_type, log);
        }
        ensure!(success, "Failed to build the '{}' shader", shader_type);

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

struct Buffers {
    pub va: glow::NativeVertexArray,
    pub positions: glow::NativeBuffer,
    pub texcoords: glow::NativeBuffer,
}

fn create_buffers(gl: &glow::Context, program: glow::Program) -> Result<Buffers> {
    let get_attrib_location = |name: &str| unsafe { gl.get_attrib_location(program, name) }
        .with_context(|| format!("Could not get '{}' attrib location", name));

    let create_buffer = || match unsafe { gl.create_buffer() } {
        Ok(buffer) => Ok(buffer),
        Err(e) => bail!("Could not create a buffer: {}", e),
    };

    let position_attrib_index = get_attrib_location("position")?;
    let texcoord_attrib_index = get_attrib_location("texcoord")?;

    // Vertex Array describes the data layout
    let vao = match unsafe { gl.create_vertex_array() } {
        Ok(buffer) => buffer,
        Err(e) => bail!("Could not create a vertex array: {}", e),
    };
    unsafe { gl.bind_vertex_array(Some(vao)) };

    let position_buffer = create_buffer()?;
    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buffer)) };
    unsafe { gl.enable_vertex_attrib_array(position_attrib_index) };
    unsafe { gl.vertex_attrib_pointer_f32(
        position_attrib_index,
        (size_of::<Position>() / size_of::<f32>()) as i32,
        glow::FLOAT,
        false,
        size_of::<Position>() as i32,
        0, // Offset into the currently bound buffer
    ) };

    let texcoord_buffer = create_buffer()?;
    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoord_buffer)) };
    unsafe { gl.enable_vertex_attrib_array(texcoord_attrib_index) };
    unsafe { gl.vertex_attrib_pointer_f32(
        texcoord_attrib_index,
        (size_of::<TexCoord>() / size_of::<f32>()) as i32,
        glow::FLOAT,
        false,
        size_of::<TexCoord>() as i32,
        0, // Offset into the currently bound buffer
    ) };

    Ok(Buffers {
        va: vao,
        positions: position_buffer,
        texcoords: texcoord_buffer,
    })
}

fn update_buffers(gl: &glow::Context, buffers: &Buffers, points: &Vec<Position>) -> Result<i32> {
    let vertices = points.len() * 6;

    let mut positions: Vec<Position> = Vec::with_capacity(vertices);
    let mut texcoords: Vec<TexCoord> = Vec::with_capacity(vertices);

    for point in points {
        let a = Position::new(point[0] - 0.1, point[1] - 0.1, point[2]);
        let b = Position::new(point[0] + 0.1, point[1] - 0.1, point[2]);
        let c = Position::new(point[0] + 0.1, point[1] + 0.1, point[2]);
        let d = Position::new(point[0] - 0.1, point[1] + 0.1, point[2]);

        positions.extend_from_slice(&[
            a, b, c,
            c, d, a,
        ]);

        texcoords.extend_from_slice(&[
            TexCoord::new(-1.0, -1.0), TexCoord::new(1.0, -1.0), TexCoord::new(1.0, 1.0),
            TexCoord::new(1.0, 1.0), TexCoord::new(-1.0, 1.0), TexCoord::new(-1.0, -1.0),
        ]);
    }

    let positions_u8: &[u8] = unsafe { slice::from_raw_parts(
        positions.as_ptr() as *const u8,
        size_of::<Position>() * vertices)
    };

    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffers.positions)) };
    unsafe { gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, positions_u8, glow::DYNAMIC_DRAW) };

    let texcoords_u8: &[u8] = unsafe { slice::from_raw_parts(
        texcoords.as_ptr() as *const u8,
        size_of::<TexCoord>() * vertices)
    };

    unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffers.texcoords)) };
    unsafe { gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, texcoords_u8, glow::DYNAMIC_DRAW) };

    Ok(vertices as i32)
}

fn delete_buffers(gl: &glow::Context, buffers: Buffers) {
    unsafe { gl.delete_vertex_array(buffers.va) };
    unsafe { gl.delete_buffer(buffers.positions) };
    unsafe { gl.delete_buffer(buffers.texcoords) };
}

fn set_uniform(gl: &glow::Context, program: glow::NativeProgram, name: &str, value: f32) {
    let uniform_location = unsafe { gl.get_uniform_location(program, name) };
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    unsafe { gl.uniform_1_f32(uniform_location.as_ref(), value) }
}
