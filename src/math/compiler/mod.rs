mod function;
mod expression;
mod var;

use super::*;
use self::var::*;
use self::function::*;
use self::expression::*;
use llvm;
use llvm::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::ptr;
use std::slice;
use std::ffi::{CStr, CString};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {}

pub unsafe fn compile(program: &Program, out_file: &mut File) -> Result<String, Error> {
    let (llvm_ctx, llvm_module) = synthesise(&program)?;

    let c = llvm::core::LLVMPrintModuleToString(llvm_module);
    let llvm_ir = CStr::from_ptr(c).to_string_lossy().into_owned();
    llvm::core::LLVMDisposeMessage(c);

    emit(llvm_module, out_file);

    llvm::core::LLVMDisposeModule(llvm_module);
    llvm::core::LLVMContextDispose(llvm_ctx);
    // FIXME: Error handling.
    return Ok(llvm_ir);
}

unsafe fn emit<W>(llvm_module: LLVMModuleRef, mut out: W)
where
    W: Write,
{
    llvm::target::LLVM_InitializeNativeTarget();
    llvm::target::LLVM_InitializeNativeAsmPrinter();
    llvm::target::LLVM_InitializeNativeAsmParser();

    let llvm_triple = CString::from_raw(llvm::target_machine::LLVMGetDefaultTargetTriple());
    let mut llvm_target = mem::uninitialized();
    assert_eq!(
        llvm::target_machine::LLVMGetTargetFromTriple(
            llvm_triple.as_ptr(),
            &mut llvm_target,
            ptr::null_mut(),
        ),
        0
    );

    let llvm_empty_name = llvm_name("");
    let llvm_target_machine = llvm::target_machine::LLVMCreateTargetMachine(
        llvm_target,
        llvm_triple.as_ptr(),
        llvm_empty_name.as_ptr(),
        llvm_empty_name.as_ptr(),
        llvm::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
        llvm::target_machine::LLVMCodeModel::LLVMCodeModelDefault,
    );

    let mut llvm_mem_buf: LLVMMemoryBufferRef = mem::uninitialized();
    llvm::target_machine::LLVMTargetMachineEmitToMemoryBuffer(
        llvm_target_machine,
        llvm_module,
        llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
        ptr::null_mut(),
        &mut llvm_mem_buf,
    );

    let llvm_out = slice::from_raw_parts(
        llvm::core::LLVMGetBufferStart(llvm_mem_buf) as *const _,
        llvm::core::LLVMGetBufferSize(llvm_mem_buf) as usize,
    );
    out.write_all(llvm_out).unwrap();
    llvm::core::LLVMDisposeMemoryBuffer(llvm_mem_buf);
    llvm::target_machine::LLVMDisposeTargetMachine(llvm_target_machine);
}

unsafe fn synthesise(program: &Program) -> Result<(*mut llvm::LLVMContext, LLVMModuleRef), Error> {
    let llvm_ctx = llvm::core::LLVMContextCreate();
    let llvm_module = llvm::core::LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
    let llvm_builder = llvm::core::LLVMCreateBuilderInContext(llvm_ctx);

    let mut llvm_functions = HashMap::new();

    for statement in &program.statements.0 {
        if let &Statement::FnDefinition(ref name, ref params, ref expr) = statement {
            let function = Function::new(name, Some(params), None, None, expr);
            let llvm_function =
                function.synthesise(llvm_ctx, llvm_module, llvm_builder, &llvm_functions);
            llvm_functions.insert(name.clone(), llvm_function);
        }
    }

    // FIXME: Construct a string type, don't create an entire global string.
    let format_string_name = llvm_name("integer_format_string");
    let format_string = llvm_name("%d");
    let llvm_format_string = llvm::core::LLVMBuildGlobalString(
        llvm_builder,
        format_string.as_ptr(),
        format_string_name.as_ptr(),
    );
    let llvm_string_type = llvm::core::LLVMTypeOf(llvm_format_string);
    let llvm_i32_type = llvm::core::LLVMInt32TypeInContext(llvm_ctx);
    let params_types = &mut [llvm_string_type];
    let llvm_fn_type = llvm::core::LLVMFunctionType(llvm_i32_type, params_types.as_mut_ptr(), 1, 0);
    let llvm_fn_name = llvm_name("printf");
    let llvm_printf_fn =
        llvm::core::LLVMAddFunction(llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type);
    llvm_functions.insert(Name::new("printf"), llvm_printf_fn);

    let main_prints = &program.outputs;
    let main_assigns = program
        .statements
        .0
        .iter()
        .filter_map(|statement| match statement {
            &Statement::VarAssignment(ref name, ref expr) => Some((name, expr)),
            _ => None,
        })
        .collect();
    let main_function_name = Name::new("main");
    let main_function = Function::new(
        &main_function_name,
        None,
        Some(main_assigns),
        Some(&main_prints),
        &Expression::Operand(Operand::I64(0)),
    );
    main_function.synthesise(llvm_ctx, llvm_module, llvm_builder, &llvm_functions);

    llvm::core::LLVMDisposeBuilder(llvm_builder);
    // FIXME: Error handling.
    return Ok((llvm_ctx, llvm_module));
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}
