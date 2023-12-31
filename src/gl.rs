use std::{
    ffi::{CStr, CString},
    ptr::{null, null_mut},
};

#[inline]
pub fn compile_shader(shader_type: u32, source: &str) -> u32 {
    let mut result = 0;

    unsafe {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &CString::new(source).unwrap().as_ptr(), null());
        gl::CompileShader(shader);

        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut result);
        if (result as u8) == gl::FALSE {
            let mut length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
            let mut log = Vec::with_capacity(length as usize);
            gl::GetShaderInfoLog(shader, length, null_mut(), log.as_mut_ptr());
            let log = CStr::from_ptr(log.as_ptr());
            let log_str = log.to_str().unwrap();
            panic!("{:#?}", log_str);
        }

        shader
    }
}

#[inline]
pub fn create_program(shaders: &[u32]) -> u32 {
    unsafe {
        let program = gl::CreateProgram();
        for i in shaders {
            gl::AttachShader(program, *i);
        }

        gl::LinkProgram(program);

        let mut result = 0;

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut result);

        if (result as u8) == gl::FALSE {
            let mut length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
            let mut log = Vec::with_capacity(length as usize);
            gl::GetProgramInfoLog(program, length, null_mut(), log.as_mut_ptr());
            let log = CStr::from_ptr(log.as_ptr());
            let log_str = log.to_str().unwrap();
            panic!("{:#?}", log_str);
        }

        program
    }
}
