#![feature(rustc_private)]

extern crate rustc_codegen_ssa;

#[allow(unused_extern_crates)]
extern crate rustc_driver;

use rustc_codegen_ssa::traits::CodegenBackend;

#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    todo!("do the codegen backend")
}
