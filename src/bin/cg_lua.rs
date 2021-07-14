#![feature(rustc_private, once_cell)]

const BUG_REPORT_URL: &str = "TODO";

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_target;

use rustc_interface::interface;
use rustc_session::{config::ErrorOutputType, early_error};
use rustc_target::spec::PanicStrategy;
use std::lazy::SyncLazy;
use std::panic;

static DEFAULT_HOOK: SyncLazy<Box<dyn Fn(&panic::PanicInfo<'_>) + Sync + Send + 'static>> =
    SyncLazy::new(|| {
        let hook = panic::take_hook();
        panic::set_hook(Box::new(|info| {
            // Invoke the default handler, which prints the actual panic message and optionally a backtrace
            (*DEFAULT_HOOK)(info);

            // Separate the output with an empty line
            eprintln!();

            // Print the ICE message
            rustc_driver::report_ice(info, BUG_REPORT_URL);
        }));
        hook
    });

pub struct LuaPassesCallback;

impl rustc_driver::Callbacks for LuaPassesCallback {
    fn config(&mut self, config: &mut interface::Config) {
        // If a --prints=... option has been given, we don't print the "total"
        // time because it will mess up the --prints output. See #64339.

        config.opts.cg.panic = Some(PanicStrategy::Abort);
        config.opts.debugging_opts.panic_abort_tests = true;
        config.opts.maybe_sysroot = Some(config.opts.maybe_sysroot.clone().unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned()
        }));
    }
}

fn main() {
    rustc_driver::init_rustc_env_logger();
    SyncLazy::force(&DEFAULT_HOOK);
    let mut callbacks = LuaPassesCallback;
    let exit_code = rustc_driver::catch_with_exit_code(|| {
        let args = std::env::args_os()
            .enumerate()
            .map(|(i, arg)| {
                arg.into_string().unwrap_or_else(|arg| {
                    early_error(
                        ErrorOutputType::default(),
                        &format!("Argument {} is not valid Unicode: {:?}", i, arg),
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut run_compiler = rustc_driver::RunCompiler::new(&args, &mut callbacks);
        run_compiler.set_make_codegen_backend(Some(Box::new(move |_| {
            rustc_codegen_lua::__rustc_codegen_backend()
        })));
        run_compiler.run()
    });

    std::process::exit(exit_code)
}
