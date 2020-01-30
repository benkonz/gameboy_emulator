use crate::webgl_rendering_context::WebGLRenderingContext;
use crate::webgl_rendering_context::*;

type Gl = WebGLRenderingContext;

pub fn compile_shader(gl: &Gl, shader_type: GLenum, source: &str) -> Result<WebGLShader, String> {
    let shader = gl.create_shader(shader_type).unwrap();
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    let compiled = gl.get_shader_parameter(&shader, Gl::COMPILE_STATUS);

    if compiled == stdweb::Value::Bool(false) {
        let error = gl.get_shader_info_log(&shader);
        Err(error.unwrap_or_else(|| "Unknown compilation error".to_string()))
    } else {
        Ok(shader)
    }
}

pub fn link_program(gl: &Gl, vert_shader: &WebGLShader, frag_shader: &WebGLShader) -> WebGLProgram {
    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, vert_shader);
    gl.attach_shader(&shader_program, frag_shader);
    gl.link_program(&shader_program);
    shader_program
}
