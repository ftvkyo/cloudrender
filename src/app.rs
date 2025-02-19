use anyhow::{bail, ensure, Context, Result};
use cgmath::{Matrix, Matrix4};
use glow::HasContext;

use crate::{camera::AppCamera, Position};

const SHADER_VERTEX: &'_ str = include_str!("shader/vertex.glsl");
const SHADER_FRAGMENT: &'_ str = include_str!("shader/fragment.glsl");

struct AppBuffers {
    pub va: glow::NativeVertexArray,
    pub positions: glow::NativeBuffer,
    pub elements: glow::NativeBuffer,
}

impl AppBuffers {
    pub fn new(gl: &glow::Context, program: glow::Program) -> Result<Self> {
        let get_attrib_location = |name: &str| {
            unsafe { gl.get_attrib_location(program, name) }
                .with_context(|| format!("Could not get '{}' attrib location", name))
        };

        let create_buffer = || match unsafe { gl.create_buffer() } {
            Ok(buffer) => Ok(buffer),
            Err(e) => bail!("Could not create a buffer: {}", e),
        };

        let position_attrib_index = get_attrib_location("position")?;

        // Vertex Array describes the data layout
        let vao = match unsafe { gl.create_vertex_array() } {
            Ok(buffer) => buffer,
            Err(e) => bail!("Could not create a vertex array: {}", e),
        };
        unsafe { gl.bind_vertex_array(Some(vao)) };

        let position_buffer = create_buffer()?;
        unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buffer)) };
        unsafe { gl.enable_vertex_attrib_array(position_attrib_index) };
        unsafe {
            gl.vertex_attrib_pointer_f32(
                position_attrib_index,
                (size_of::<Position>() / size_of::<f32>()) as i32,
                glow::FLOAT,
                false,
                size_of::<Position>() as i32,
                0, // Offset into the currently bound buffer
            )
        };

        let element_array_buffer = create_buffer()?;
        unsafe { gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_array_buffer)) };

        Ok(Self {
            va: vao,
            positions: position_buffer,
            elements: element_array_buffer,
        })
    }

    pub fn update_elements(&self, gl: &glow::Context, points: usize) -> Result<i32> {
        let vertices = points * 6;

        let mut elements: Vec<u32> = Vec::with_capacity(vertices);

        for point in 0..points {
            let p = point as u32 * 4;
            elements.extend_from_slice(&[p, p + 1, p + 2, p + 2, p + 3, p]);
        }

        let elements_u8: &[u8] = unsafe {
            ::core::slice::from_raw_parts(
                elements.as_ptr() as *const u8,
                size_of::<u32>() * vertices,
            )
        };

        unsafe { gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.elements)) };
        unsafe {
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, elements_u8, glow::DYNAMIC_DRAW)
        };

        Ok(vertices as i32)
    }

    pub fn update_positions(&self, gl: &glow::Context, points: &[Position]) {
        let vertices = points.len() * 4;

        let mut positions: Vec<Position> = Vec::with_capacity(vertices);

        for point in points {
            let a = Position::new(point[0] - 0.1, point[1] - 0.1, point[2]);
            let b = Position::new(point[0] + 0.1, point[1] - 0.1, point[2]);
            let c = Position::new(point[0] + 0.1, point[1] + 0.1, point[2]);
            let d = Position::new(point[0] - 0.1, point[1] + 0.1, point[2]);

            positions.extend_from_slice(&[a, b, c, d]);
        }

        let positions_u8: &[u8] = unsafe {
            ::core::slice::from_raw_parts(
                positions.as_ptr() as *const u8,
                size_of::<Position>() * vertices,
            )
        };

        unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.positions)) };
        unsafe { gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, positions_u8, glow::DYNAMIC_DRAW) };
    }
}

fn create_program(gl: &glow::Context) -> Result<glow::Program> {
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
        unsafe {
            gl.compile_shader(shader);
        }

        let success = unsafe { gl.get_shader_compile_status(shader) };
        if !success || cfg!(debug_assertions) {
            let log = unsafe { gl.get_shader_info_log(shader) };
            if !log.is_empty() {
                println!("Shader '{}' info log:\n{}", shader_type, log);
            }
        }
        ensure!(success, "Failed to build the '{}' shader", shader_type);

        unsafe { gl.attach_shader(program, shader) };

        shaders.push(shader);
    }

    unsafe { gl.link_program(program) };
    if !unsafe { gl.get_program_link_status(program) } {
        bail!("{}", unsafe { gl.get_program_info_log(program) });
    }

    for shader in shaders {
        unsafe { gl.detach_shader(program, shader) };
        unsafe { gl.delete_shader(shader) };
    }

    Ok(program)
}

pub struct App {
    #[allow(dead_code)] // Even if not accessed, this needs to outlive all GL operations
    gl_ctx: sdl3::video::GLContext,
    window: sdl3::video::Window,
    event_pump: sdl3::EventPump,

    gl: glow::Context,
    program: glow::Program,

    buffers: AppBuffers,
}

impl App {
    pub fn init(w: u32, h: u32) -> Result<Self> {
        let sdl = sdl3::init()?;
        let video = sdl.video()?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_flags().forward_compatible().set();

        let window = video
            .window("rust-sdl3 demo", w, h)
            .position_centered()
            .opengl()
            .build()?;

        // This needs to be created before function loading.
        // This should only be dropped after we are done with any GL.
        let gl_ctx = window.gl_create_context()?;

        let gl_loader = |s: &str| match video.gl_get_proc_address(s) {
            Some(f) => f as *const ::core::ffi::c_void,
            None => ::std::ptr::null(),
        };

        let gl = unsafe { glow::Context::from_loader_function(gl_loader) };

        unsafe { gl.enable(glow::BLEND) };
        unsafe { gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA) };

        let event_pump = sdl.event_pump()?;

        let program = create_program(&gl)?;
        unsafe { gl.use_program(Some(program)) };

        let buffers = AppBuffers::new(&gl, program)?;

        unsafe { gl.clear_color(0.0, 0.0, 0.0, 1.0) };

        Ok(Self {
            gl_ctx,
            window,
            event_pump,

            gl,
            program,
            buffers,
        })
    }

    pub fn poll_iter(&mut self) -> sdl3::event::EventPollIterator {
        self.event_pump.poll_iter()
    }

    pub fn update_uniforms(&self, model: &Matrix4<f32>, camera: &AppCamera) -> Result<()> {
        let get_uniform_location =
            |name: &str| unsafe { self.gl.get_uniform_location(self.program, name) };

        let model_f32 = unsafe {
            core::slice::from_raw_parts(
                model.as_ptr(),
                size_of_val(model) / size_of_val(&model.x.x),
            )
        };

        let model_uniform_location = get_uniform_location("model");
        unsafe {
            self.gl
                .uniform_matrix_4_f32_slice(model_uniform_location.as_ref(), false, model_f32)
        };

        let view = camera.view();
        let view_f32 = unsafe {
            core::slice::from_raw_parts(view.as_ptr(), size_of_val(&view) / size_of_val(&view.x.x))
        };

        let view_uniform_location = get_uniform_location("view");
        unsafe {
            self.gl
                .uniform_matrix_4_f32_slice(view_uniform_location.as_ref(), false, view_f32)
        };

        let projection = camera.projection();
        let projection_f32 = unsafe {
            core::slice::from_raw_parts(
                projection.as_ptr(),
                size_of_val(&projection) / size_of_val(&projection.x.x),
            )
        };

        let projection_uniform_location = get_uniform_location("projection");
        unsafe {
            self.gl.uniform_matrix_4_f32_slice(
                projection_uniform_location.as_ref(),
                false,
                projection_f32,
            )
        };

        Ok(())
    }

    pub fn render_frame(&self, points: &[Position]) -> Result<()> {
        let vertices = self.buffers.update_elements(&self.gl, points.len())?;
        self.buffers.update_positions(&self.gl, points);

        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
        unsafe {
            self.gl
                .draw_elements(glow::TRIANGLES, vertices, glow::UNSIGNED_INT, 0)
        };
        self.window.gl_swap_window();

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffer(self.buffers.positions) };
        unsafe { self.gl.delete_buffer(self.buffers.elements) };
        unsafe { self.gl.delete_vertex_array(self.buffers.va) };
        unsafe { self.gl.delete_program(self.program) };
    }
}
