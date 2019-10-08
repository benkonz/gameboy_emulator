use gl;
use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;

pub struct Shader {
    pub program: GLuint,
}

impl Shader {
    pub fn new(vertex_source: &str, fragment_source: &str) -> Shader {
        let program = unsafe {
            let vertex_shader = compile_shader(vertex_source, gl::VERTEX_SHADER);
            let fragment_shader = compile_shader(fragment_source, gl::FRAGMENT_SHADER);
            let program = link_shaders(vertex_shader, fragment_shader);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            program
        };

        Shader { program }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

unsafe fn compile_shader(source: &str, shader_type: GLenum) -> GLuint {
    let shader = gl::CreateShader(shader_type);
    let source = CString::new(source.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1);
        gl::GetShaderInfoLog(
            shader,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );

        panic!(String::from_utf8(buf));
    }

    shader
}

unsafe fn link_shaders(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vertex_shader);
    gl::AttachShader(program, fragment_shader);
    gl::LinkProgram(program);
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        //subtract 1 to remove the null character
        buf.set_len((len as usize) - 1);
        gl::GetProgramInfoLog(
            program,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );

        panic!(String::from_utf8(buf));
    }

    program
}
