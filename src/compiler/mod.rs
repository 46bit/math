mod function;
mod expression;
mod param;
mod func;
mod operations;
mod program;
mod io;

use super::*;
use self::param::*;
use self::function::*;
use self::expression::*;
use self::func::*;
use self::operations::*;
use self::program::*;
use self::io::*;
use llvm;
use llvm::prelude::*;
use llvm::core::*;
use libc;
use tempfile::NamedTempFile;
use std::ptr;
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use std::process::Command;
use std::slice;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Emit {
    IR(Option<PathBuf>),
    Object(PathBuf),
    Binary(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unknown,
}

pub unsafe fn compile(program: &Program, emit: Emit) -> Result<String, Error> {
    match emit {
        Emit::IR(None) => synthesise(program, None),
        Emit::IR(Some(pathbuf)) => synthesise(program, Some(pathbuf.as_path())),
        Emit::Object(pathbuf) => {
            let ir = synthesise(program, None)?;
            objectify(&ir, pathbuf.as_path())?;
            Ok(ir)
        }
        Emit::Binary(pathbuf) => {
            let ir = synthesise(program, None)?;
            let tempfile = NamedTempFile::new().unwrap();
            objectify(&ir, tempfile.path())?;
            link(tempfile.path(), pathbuf.as_path())?;
            drop(tempfile);
            Ok(ir)
        }
    }
}

unsafe fn synthesise(program: &Program, ir_path: Option<&Path>) -> Result<String, Error> {
    let ctx = LLVMContextCreate();
    let module = LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
    let builder = LLVMCreateBuilderInContext(ctx);

    define_sscanf(ctx, module);
    define_printf(ctx, module);
    define_saturating_div(ctx, module, builder);

    let input_function = define_input(ctx, module, builder, program.inputs.clone());
    let output_function = define_output(ctx, module, builder, program.outputs.clone());

    let mut defines = vec![];
    let mut assigns = vec![];
    for statement in &program.statements.0 {
        match statement {
            &Statement::FnDefinition(ref name, ref params, ref expr) => {
                defines.push((name.clone(), params.clone(), expr.clone()));
            }
            &Statement::VarAssignment(ref name, ref expression) => {
                assigns.push((name.clone(), expression.clone()));
            }
        }
    }

    let mut functions = HashMap::new();
    for (name, params, expr) in defines {
        let function = synthesise_function(ctx, module, builder, &name, &params, &expr, &functions);
        functions.insert(name.clone(), function);
    }

    let run_params = classify_parameters(&program.inputs, &program.outputs);
    let run_function = define_run(
        ctx,
        module,
        builder,
        run_params.clone(),
        assigns,
        &functions,
    );
    define_main(
        ctx,
        module,
        builder,
        run_params,
        input_function,
        run_function,
        output_function,
        program.inputs.len() as u64,
    );

    LLVMDisposeBuilder(builder);
    let ir_ptr = LLVMPrintModuleToString(module);
    let ir = CStr::from_ptr(ir_ptr).to_string_lossy().into_owned();
    LLVMDisposeMessage(ir_ptr);
    LLVMDisposeModule(module);
    LLVMContextDispose(ctx);

    if let Some(path) = ir_path {
        let mut f = File::create(path).unwrap();
        f.write_all(ir.as_bytes()).unwrap();
        drop(f);
    }
    Ok(ir)
}

unsafe fn objectify(llvm_ir: &String, object_path: &Path) -> Result<(), Error> {
    let llvm_ctx = LLVMContextCreate();
    let llvm_ir_str = llvm_name(llvm_ir);
    let llvm_ir_buffer_name = llvm_name("llvm_ir_buffer");
    let llvm_ir_buffer = LLVMCreateMemoryBufferWithMemoryRange(
        llvm_ir_str.as_ptr(),
        llvm_ir_str.as_bytes().len(),
        llvm_ir_buffer_name.as_ptr(),
        0,
    );
    let mut llvm_module = ptr::null_mut();
    let mut errors = ptr::null_mut();
    let return_code = llvm::ir_reader::LLVMParseIRInContext(
        llvm_ctx,
        llvm_ir_buffer,
        &mut llvm_module,
        &mut errors,
    );
    if errors != ptr::null_mut() {
        println!("{}", CString::from_raw(errors).to_str().unwrap());
    }
    assert_eq!(return_code, 0);

    llvm::target::LLVM_InitializeNativeTarget();
    llvm::target::LLVM_InitializeNativeAsmPrinter();
    llvm::target::LLVM_InitializeNativeAsmParser();

    let llvm_triple_s = llvm::target_machine::LLVMGetDefaultTargetTriple();
    let llvm_triple = CStr::from_ptr(llvm_triple_s);
    let mut llvm_target = ptr::null_mut();
    assert_eq!(
        llvm::target_machine::LLVMGetTargetFromTriple(
            llvm_triple.as_ptr(),
            &mut llvm_target,
            ptr::null_mut(),
        ),
        0
    );

    let llvm_target_machine = llvm::target_machine::LLVMCreateTargetMachine(
        llvm_target,
        llvm_triple.as_ptr(),
        ptr::null(),
        ptr::null(),
        llvm::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
        llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
        llvm::target_machine::LLVMCodeModel::LLVMCodeModelDefault,
    );
    assert_ne!(llvm_target_machine, ptr::null_mut());

    let mut llvm_mem_buf: LLVMMemoryBufferRef = ptr::null_mut();
    assert_eq!(
        llvm::target_machine::LLVMTargetMachineEmitToMemoryBuffer(
            llvm_target_machine,
            llvm_module,
            llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
            ptr::null_mut(),
            &mut llvm_mem_buf,
        ),
        0
    );
    let llvm_out = slice::from_raw_parts(
        LLVMGetBufferStart(llvm_mem_buf) as *const _,
        LLVMGetBufferSize(llvm_mem_buf) as usize,
    );

    let mut object_file = File::create(object_path).unwrap();
    object_file.write_all(llvm_out).unwrap();
    drop(object_file);

    libc::free(llvm_triple_s as *mut libc::c_void);
    LLVMDisposeMemoryBuffer(llvm_mem_buf);
    llvm::target_machine::LLVMDisposeTargetMachine(llvm_target_machine);
    LLVMDisposeModule(llvm_module);
    LLVMContextDispose(llvm_ctx);

    Ok(())
}

fn link(object_path: &Path, binary_path: &Path) -> Result<(), Error> {
    assert!(
        Command::new("cc")
            .arg("-o")
            .arg(binary_path)
            .arg(object_path)
            .spawn()
            .expect("could not invoke cc for linking")
            .wait()
            .unwrap()
            .success()
    );
    Ok(())
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser;
    use rand::thread_rng;
    use quickcheck::{QuickCheck, StdGen};

    fn synthesises_successfully_property(program: Program) -> bool {
        eprintln!("{}\n---", program);
        unsafe { synthesise(&program).is_ok() }
    }

    #[test]
    fn synthesises_successfully() {
        // QuickCheck's default size creates infeasibly vast statements, and beyond some
        // point they stop exploring novel code paths. This does a much better job of
        // exploring potential edgecases.
        for size in 1..11 {
            let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), size));
            qc.quickcheck(synthesises_successfully_property as fn(Program) -> bool);
        }
    }

    #[test]
    fn can_synthesise_with_no_inputs_or_outputs() {
        unsafe {
            synthesise(&parse(b"inputs; outputs;")).unwrap();
        }
    }

    #[test]
    fn can_synthesise_inputs_into_outputs() {
        unsafe {
            synthesise(&parse(b"inputs a; outputs a;")).unwrap();
        }
    }

    fn parse(s: &[u8]) -> Program {
        parser::parse(s).unwrap()
    }
}
