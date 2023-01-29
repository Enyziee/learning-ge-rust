extern crate gl;
extern crate glfw;

use std::{
    ffi::{c_void, CString},
    mem::{size_of, size_of_val},
    ptr::null,
};

use glfw::{Action, Context, Key};

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

    let vertex_shader_src = CString::new(
        "
        #version 430 core
        layout (location = 0) in vec3 aPos;
        void main()
        {
            gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
        }",
    )
    .unwrap();

    let fragment_shader_src = CString::new(
        "
        #version 430 core
        out vec4 FragColor;
        void main()
        {
           FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
        }",
    )
    .unwrap();

    let shader_program = unsafe {
        // Create Vertex Shader
        let vertex_shader: u32 = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &vertex_shader_src.as_ptr(), null());
        gl::CompileShader(vertex_shader);

        // Create Fragment Shader
        let fragment_shader: u32 = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &fragment_shader_src.as_ptr(), null());
        gl::CompileShader(fragment_shader);

        // Create Shader Program
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
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
            gl::UseProgram(shader_program);
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
