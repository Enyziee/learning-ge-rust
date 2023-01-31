use std::{mem::size_of_val, os::raw::c_void};

use gl::types::*;

pub struct Buffer {
    id: u32,
    buffer_type: GLenum,
}

impl Buffer {
    pub unsafe fn new(buffer_type: GLenum) -> Self {
        let mut buffer = Self { id: 0, buffer_type };

        gl::GenBuffers(1, &mut buffer.id);

        return buffer;
    }
}

impl Buffer {
    pub unsafe fn bind(&self) {
        gl::BindBuffer(self.buffer_type, self.id);
    }
}

impl Buffer {
    pub unsafe fn set_data<D>(&self, data: &[D], usage: GLuint) {
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

pub struct VertexArray {
    pub id: u32,
}

impl VertexArray {
    pub unsafe fn new() -> Self {
        let mut vao = Self { id: 0 };
        gl::GenVertexArrays(1, &mut vao.id);

        return vao;
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.id);
    }

    pub unsafe fn set_layout(
        &self,
        location: u32,
        count: i32,
        data_type: GLenum,
        normalized: GLboolean,
        stride: i32,
    ) {
        gl::VertexAttribPointer(
            location,
            count,
            data_type,
            normalized,
            stride,
            0 as *const c_void,
        );
        gl::EnableVertexAttribArray(location);
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.id);
        }
    }
}
