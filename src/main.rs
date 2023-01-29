extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use std::{
    ffi::{c_void, CString},
    mem::{size_of, size_of_val},
    ptr::null,
    string::FromUtf8Error,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Error while compiling shader: {0}")]
    CompilationError(String),
    #[error("Error while linking shaders: {0}")]
    LinkingError(String),
    #[error{"{0}"}]
    Utf8Error(#[from] FromUtf8Error),
    #[error{"{0}"}]
    NulError(#[from] std::ffi::NulError),
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(854, 480, "OpenGL in Rust", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_resizable(false);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    // SHADERS

    let vertex_shader_src = "
        #version 430 core
        layout (location = 0) in vec3 aPos;
        void main()
        {
            gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
        }";

    let fragment_shader_src = "
        #version 430 core
        out vec4 FragColor;
        void main()
        {
           FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
        }";

    struct Shader {
        id: u32,
    }

    impl Shader {
        unsafe fn new(shader_source: &str, shader_type: GLenum) -> Result<Self, ShaderError> {
            let shader = Self {
                id: gl::CreateShader(shader_type),
            };
            let shader_source = CString::new(shader_source).unwrap();

            gl::ShaderSource(shader.id, 1, &shader_source.as_ptr(), null());
            gl::CompileShader(shader.id);

            // check for shader compilation errors
            let mut success: GLint = 0;
            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);

            if success == 1 {
                Ok(shader)
            } else {
                let mut error_log_size: GLint = 0;
                gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                gl::GetShaderInfoLog(
                    shader.id,
                    error_log_size,
                    &mut error_log_size,
                    error_log.as_mut_ptr() as *mut _,
                );

                error_log.set_len(error_log_size as usize);
                let log = String::from_utf8(error_log).unwrap();
                Err(ShaderError::CompilationError(log))
            }
        }
    }

    impl Drop for Shader {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteShader(self.id);
            }
        }
    }

    struct ShaderProgram {
        id: u32,
    }

    impl ShaderProgram {
        unsafe fn new(shaders: &[Shader]) -> Result<Self, ShaderError> {
            let program = Self {
                id: gl::CreateProgram(),
            };

            for shader in shaders {
                gl::AttachShader(program.id, shader.id);
            }

            gl::LinkProgram(program.id);

            let mut sucess: i32 = 0;
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut sucess);

            if sucess == 1 {
                Ok(program)
            } else {
                let mut error_log_size: i32 = 0;
                gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                gl::GetProgramInfoLog(
                    program.id,
                    error_log_size,
                    &mut error_log_size,
                    error_log.as_mut_ptr() as *mut _,
                );

                error_log.set_len(error_log_size as usize);
                let log = String::from_utf8(error_log)?;
                Err(ShaderError::LinkingError(log))
            }
        }
    }

    impl ShaderProgram {
        unsafe fn apply(&self) {
            gl::UseProgram(self.id);
        }
    }

    impl Drop for ShaderProgram {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteShader(self.id);
            }
        }
    }

    let shader_program = unsafe {
        let vertex_shader = Shader::new(&vertex_shader_src, gl::VERTEX_SHADER)
            .expect("Failed to create Vertex Shader");
        let fragment_shader = Shader::new(&fragment_shader_src, gl::FRAGMENT_SHADER)
            .expect("Failed to create Fragment Shader");
        let shader_program = ShaderProgram::new(&[vertex_shader, fragment_shader])
            .expect("Failed to create Shader Program");

        shader_program
    };    

    type Vertex = [f32; 3];
    let vertex_data: [Vertex; 4] = [
        [0.5, 0.5, 0.0],
        [0.5, -0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    let index = [0, 1, 2, 0, 2, 3];

    unsafe {
        let mut vertex_array: u32 = 0;
        let mut element_array: u32 = 0;
        let mut vertex_buffer: u32 = 0;

        gl::GenVertexArrays(1, &mut vertex_array);
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::GenBuffers(1, &mut element_array);

        gl::BindVertexArray(vertex_array);

        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&vertex_data) as isize,
            vertex_data.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_array);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            size_of_val(&index) as isize,
            index.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>() as i32,
            0 as *const c_void,
        );
        gl::EnableVertexAttribArray(0);

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        while !window.should_close() {
            glfw.poll_events();

            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw
            shader_program.apply();
            gl::BindVertexArray(vertex_array);

            // gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::DrawElements(
                gl::TRIANGLES,
                6 as i32,
                gl::UNSIGNED_INT,
                0 as *const c_void,
            );

            window.swap_buffers();
            for (_, event) in glfw::flush_messages(&events) {
                handle_window_event(&mut window, event);
            }
        }
    };
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        _ => {}
    }
}
