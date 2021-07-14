# Lua codegen Backend for Rust

Why would you want to compile rust to lua ? because you may want to run some rust where the only interface is a Lua environnement. I have no clue if it's possible to map enough of the Rust MIR to lua or if a direct Rust to Lua transpiler is the better route, we will see

All the scaffolding for building a codegen backend is taken from [rustc_codegen_cranelift](https://github.com/bjorn3/rustc_codegen_cranelift).

## Building

To build the backend you can do 
```
./y.rs prepare
./y.rs build
```

And then you can user `$cg_lua_dir/build/cargo` to build using the codegen backend
