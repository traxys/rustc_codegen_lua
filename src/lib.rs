#![feature(rustc_private)]

extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_middle;
extern crate rustc_session;

#[allow(unused_extern_crates)]
extern crate rustc_driver;

use rustc_codegen_ssa::{traits::CodegenBackend, CodegenResults};
use rustc_data_structures::fx::FxHashMap;
use rustc_errors::ErrorReported;
use rustc_middle::{
    dep_graph::{WorkProduct, WorkProductId},
    middle::cstore::EncodedMetadata,
    ty::TyCtxt,
};
use rustc_session::{config::OutputFilenames, Session};

use std::any::Any;

#[derive(Clone)]
pub struct LuaCodegenBackend;

impl CodegenBackend for LuaCodegenBackend {
    fn init(&self, _sess: &Session) {}

    fn codegen_crate<'tcx>(
        &self,
        _tcx: TyCtxt<'tcx>,
        _metadata: EncodedMetadata,
        _need_metadata_module: bool,
    ) -> Box<dyn Any> {
        todo!("codegen crate")
    }

    fn join_codegen(
        &self,
        _ongoing_codegen: Box<dyn Any>,
        _sess: &Session,
    ) -> Result<(CodegenResults, FxHashMap<WorkProductId, WorkProduct>), ErrorReported> {
        todo!("join codegen")
    }

    fn link(
        &self,
        _sess: &Session,
        _codegen_results: CodegenResults,
        _outputs: &OutputFilenames,
    ) -> Result<(), ErrorReported> {
        todo!("link")
    }
}

#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(LuaCodegenBackend)
}
