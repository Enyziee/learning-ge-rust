use gl::types::*;
use std::{ffi::CString, string::FromUtf8Error};
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

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub unsafe fn new(shader_source: &str, shader_type: GLenum) -> Result<Self, ShaderError> {
        let shader = Self {
            id: gl::CreateShader(shader_type),
        };
        let shader_source = CString::new(shader_source).unwrap();

        gl::ShaderSource(shader.id, 1, &shader_source.as_ptr(), std::ptr::null());
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

pub struct ShaderProgram {
    pub id: u32,
}

impl ShaderProgram {
    pub unsafe fn new(shaders: &[Shader]) -> Result<Self, ShaderError> {
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
    pub unsafe fn apply(&self) {
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
