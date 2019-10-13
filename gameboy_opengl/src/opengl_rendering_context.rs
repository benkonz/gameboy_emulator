#![allow(dead_code, unused_parens, unused_imports)]

pub use self::Gles2 as Gl;
include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
