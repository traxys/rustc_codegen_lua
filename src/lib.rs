#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_symbol_mangling;

#[allow(unused_extern_crates)]
extern crate rustc_driver;

use rustc_ast::expand::allocator::AllocatorKind;
use rustc_codegen_ssa::{
    back::{
        lto::{LtoModuleCodegen, SerializedModule, ThinModule},
        write::{CodegenContext, FatLTOInput, ModuleConfig, TargetMachineFactoryFn},
    },
    base::codegen_crate,
    traits::{
        CodegenBackend, ExtraBackendMethods, ModuleBufferMethods, ThinBufferMethods,
        WriteBackendMethods,
    },
    CodegenResults, CompiledModule, ModuleCodegen,
};
use rustc_data_structures::fx::FxHashMap;
use rustc_errors::{ErrorReported, Handler};
use rustc_middle::{
    dep_graph::{WorkProduct, WorkProductId},
    middle::cstore::EncodedMetadata,
    ty::TyCtxt,
};
use rustc_session::{
    config::{OptLevel, OutputFilenames},
    Session,
};
use rustc_span::{fatal_error::FatalError, Symbol};

use std::any::Any;
use std::sync::Arc;

#[derive(Clone)]
pub struct LuaCodegenBackend;

impl CodegenBackend for LuaCodegenBackend {
    fn init(&self, _sess: &Session) {}

    fn codegen_crate<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn Any> {
        let target_cpu = target_cpu(tcx.sess);
        let res = codegen_crate(
            self.clone(),
            tcx,
            target_cpu.to_string(),
            metadata,
            need_metadata_module,
        );

        rustc_symbol_mangling::test::report_symbol_names(tcx);

        Box::new(res)
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

pub struct LuaContext {}

impl ExtraBackendMethods for LuaCodegenBackend {
    fn new_metadata<'tcx>(&self, _tcx: TyCtxt<'tcx>, _mod_name: &str) -> Self::Module {
        LuaContext {}
    }

    fn write_compressed_metadata<'tcx>(
        &self,
        _tcx: TyCtxt<'tcx>,
        _metadata: &EncodedMetadata,
        _module: &mut Self::Module,
    ) {
        unimplemented!("write metadata")
    }

    fn codegen_allocator<'tcx>(
        &self,
        _tcx: TyCtxt<'tcx>,
        _mods: &mut Self::Module,
        _kind: AllocatorKind,
        _has_alloc_error_handler: bool,
    ) {
        unimplemented!("allocator codegen")
    }

    fn compile_codegen_unit<'tcx>(
        &self,
        _tcx: TyCtxt<'tcx>,
        _cgu_name: Symbol,
    ) -> (ModuleCodegen<Self::Module>, u64) {
        unimplemented!("compile codegen units")
    }

    fn target_machine_factory(
        &self,
        _sess: &Session,
        _opt_level: OptLevel,
    ) -> TargetMachineFactoryFn<Self> {
        Arc::new(|_| Ok(()))
    }

    fn target_cpu<'b>(&self, _sess: &'b Session) -> &'b str {
        unimplemented!()
    }

    fn tune_cpu<'b>(&self, _sess: &'b Session) -> Option<&'b str> {
        None
    }
}

pub struct ModuleBuffer;

impl ModuleBufferMethods for ModuleBuffer {
    fn data(&self) -> &[u8] {
        unimplemented!()
    }
}

pub struct ThinBuffer;

impl ThinBufferMethods for ThinBuffer {
    fn data(&self) -> &[u8] {
        unimplemented!()
    }
}

impl WriteBackendMethods for LuaCodegenBackend {
    type Module = LuaContext;
    type TargetMachine = ();
    type ModuleBuffer = ModuleBuffer;
    type Context = ();
    type ThinData = ();
    type ThinBuffer = ThinBuffer;

    fn run_fat_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<FatLTOInput<Self>>,
        _cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<LtoModuleCodegen<Self>, FatalError> {
        panic!("fat lto in lua has not a lot of meaning");
    }

    fn run_thin_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<(String, Self::ThinBuffer)>,
        _cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<(Vec<LtoModuleCodegen<Self>>, Vec<WorkProduct>), FatalError> {
        panic!("thin lto in lua has not a lot of meaning");
    }

    fn print_pass_timings(&self) {
        unimplemented!()
    }

    unsafe fn optimize(
        _cgcx: &CodegenContext<Self>,
        _diag_handler: &Handler,
        _module: &ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        Ok(())
    }

    unsafe fn optimize_thin(
        _cgcx: &CodegenContext<Self>,
        _thin: &mut ThinModule<Self>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!()
    }

    unsafe fn codegen(
        _cgcx: &CodegenContext<Self>,
        _diag_handler: &Handler,
        _module: ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
    ) -> Result<CompiledModule, FatalError> {
        todo!("write::codegen")
    }

    fn prepare_thin(_module: ModuleCodegen<Self::Module>) -> (String, Self::ThinBuffer) {
        unimplemented!()
    }

    fn serialize_module(_module: ModuleCodegen<Self::Module>) -> (String, Self::ModuleBuffer) {
        unimplemented!()
    }

    fn run_lto_pass_manager(
        _cgcx: &CodegenContext<Self>,
        _module: &ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
        _thin: bool,
    ) -> Result<(), FatalError> {
        Ok(())
    }

    fn run_link(
        _cgcx: &CodegenContext<Self>,
        _diag_handler: &Handler,
        _modules: Vec<ModuleCodegen<Self::Module>>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!("run link")
    }
}

fn handle_native(name: &str) -> &str {
    if name != "native" {
        return name;
    }

    todo!("handle native")
}

pub fn target_cpu(sess: &Session) -> &str {
    let name = sess.opts.cg.target_cpu.as_ref().unwrap_or(&sess.target.cpu);
    handle_native(name)
}

#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(LuaCodegenBackend)
}
