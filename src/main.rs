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
        .create_window(800, 600, "OpenGL in Rust", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_resizable(false);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    // SHADERS

    let vertex_shader_src = "
        #version 430 core

        layout (location = 0) in vec3 vPosition;
        layout (location = 1) in vec3 vColor;

        out vec3 color;

        uniform float xPosition;
        uniform float yPosition;

        void main()
        {
            color = vColor;
            gl_Position = vec4((vPosition.x + xPosition), (vPosition.y + yPosition), vPosition.z, 1.0);
        }";

    let fragment_shader_src = "
        #version 430 core
        in vec3 color;
        out vec4 FragColor;

        uniform float xPosition;
        uniform float yPosition;

        void main()
        {
           FragColor = vec4(color, 1.0f);
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

    struct Buffer {
        id: u32,
        buffer_type: GLenum,
    }

    impl Buffer {
        unsafe fn new(buffer_type: GLenum) -> Self {
            let mut buffer = Self { id: 0, buffer_type };

            gl::GenBuffers(1, &mut buffer.id);

            return buffer;
        }
    }

    impl Buffer {
        unsafe fn bind(&self) {
            gl::BindBuffer(self.buffer_type, self.id);
        }
    }

    impl Buffer {
        unsafe fn set_data<D>(&self, data: &[D], usage: GLuint) {
            self.bind();
            let (_, data_bytes, _) = data.align_to::<f32>();

            gl::BufferData(
                self.buffer_type,
                size_of_val(data_bytes) as isize,
                data.as_ptr() as *const c_void,
                usage,
            );
        }
    }

    impl Drop for Buffer {
        fn drop(&mut self) {
            unsafe { gl::DeleteBuffers(1, [self.id].as_mut_ptr()) }
        }
    }

    type Vertex = [f32; 3];
    // let vertex_positions: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
    let vertex_colors: [Vertex; 3] = [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];

    // let index = [0, 1, 2];

    let vertices: [Vertex; 4] = [
        [0.5, 0.5, 0.0],   // top right
        [0.5, -0.5, 0.0],  // bottom right
        [-0.5, -0.5, 0.0], // bottom left
        [-0.5, 0.5, 0.0],  // top left
    ];
    let indices = [
        // note that we start from 0!
        0, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ];

    unsafe {
        let mut vertex_array: u32 = 0;
        let index_array = Buffer::new(gl::ELEMENT_ARRAY_BUFFER);
        let vertex_buffer = Buffer::new(gl::ARRAY_BUFFER);
        let color_buffer = Buffer::new(gl::ARRAY_BUFFER);

        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);

        vertex_buffer.bind();
        vertex_buffer.set_data(&vertices, gl::STATIC_DRAW);

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>() as i32,
            0 as *const c_void,
        );
        gl::EnableVertexAttribArray(0);

        color_buffer.bind();
        color_buffer.set_data(&vertex_colors, gl::STATIC_DRAW);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>() as i32,
            0 as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        index_array.bind();
        index_array.set_data(&indices, gl::STATIC_DRAW);

        // let uniform_name: Vec<u8> = Vec::from("xPosition");
        let mut x_value = 0.0;
        let mut y_value = 0.0;

        while !window.should_close() {
            glfw.poll_events();

            // gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw
            shader_program.apply();
            gl::BindVertexArray(vertex_array);

            let uniform_name = CString::new("xPosition").unwrap();
            let uniform_location_x =
                gl::GetUniformLocation(shader_program.id, uniform_name.into_raw());

            let uniform_name = CString::new("yPosition").unwrap();
            let uniform_location_y =
                gl::GetUniformLocation(shader_program.id, uniform_name.into_raw());

            // gl::Uniform1f(uniform_location, value);

            // gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::DrawElements(
                gl::TRIANGLES,
                6 as i32,
                gl::UNSIGNED_INT,
                0 as *const c_void,
            );

            let movement = 0.02;

            window.swap_buffers();
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Right, _, Action::Repeat, _) => {
                        x_value += movement;
                        gl::Uniform1f(uniform_location_x, x_value);
                    }
                    glfw::WindowEvent::Key(Key::Left, _, Action::Repeat, _) => {
                        x_value -= movement;
                        gl::Uniform1f(uniform_location_x, x_value);
                    }
                    glfw::WindowEvent::Key(Key::Up, _, Action::Repeat, _) => {
                        y_value += movement;
                        gl::Uniform1f(uniform_location_y, y_value);
                    }
                    glfw::WindowEvent::Key(Key::Down, _, Action::Repeat, _) => {
                        y_value -= movement;
                        gl::Uniform1f(uniform_location_y, y_value);
                    }

                    _ => {}
                }

                handle_window_event(&mut window, event);
            }
        }
    };
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => unsafe {
            println!("Wireframe OFF");
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        },
        glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => unsafe {
            println!("Wireframe ON");
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        },

        _ => {}
    }
}
