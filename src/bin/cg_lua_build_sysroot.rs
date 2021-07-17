#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_target;

use std::path::PathBuf;

use rustc_interface::interface;
use rustc_session::{config::ErrorOutputType, early_error};
use rustc_target::spec::PanicStrategy;

fn find_sysroot() -> String {
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
    match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect("need to specify RUST_SYSROOT or use rustup or multirust")
            .to_owned(),
    }
}

pub struct LuaPassesCallbacks {
    use_lua: bool,
}

impl rustc_driver::Callbacks for LuaPassesCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        if !self.use_lua {
            config.opts.maybe_sysroot = Some(PathBuf::from(find_sysroot()));
            return;
        }

        config.opts.cg.panic = Some(PanicStrategy::Abort);
        config.opts.debugging_opts.panic_abort_tests = true;
        config.opts.maybe_sysroot = Some(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned(),
        );
    }
}

fn main() {
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();

    let exit_code = rustc_driver::catch_with_exit_code(|| {
        let mut use_lua = false;

        let args: Vec<_> = std::env::args_os()
            .enumerate()
            .map(|(i, arg)| {
                arg.into_string().unwrap_or_else(|arg| {
                    early_error(
                        ErrorOutputType::default(),
                        &format!("Argument {} is not valid unicode: {:?}", i, arg),
                    )
                })
            })
            .filter(|arg| {
                if arg == "--lua" {
                    use_lua = true;
                    false
                } else {
                    true
                }
            })
            .collect();

        let mut callbacks = LuaPassesCallbacks { use_lua };

        let mut run_compiler = rustc_driver::RunCompiler::new(&args, &mut callbacks);
        if use_lua {
            run_compiler.set_make_codegen_backend(Some(Box::new(move |_| {
                rustc_codegen_lua::__rustc_codegen_backend()
            })));
        }
        run_compiler.run()
    });

    std::process::exit(exit_code)
}
