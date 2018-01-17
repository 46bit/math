mod function;
mod expression;

use super::*;
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

pub unsafe fn compile(statements: &Statements) -> Result<String, Error> {
    let (llvm_ctx, llvm_module) = synthesise(&statements.0)?;

    let c = llvm::core::LLVMPrintModuleToString(llvm_module);
    let llvm_ir = CStr::from_ptr(c).to_string_lossy().into_owned();
    llvm::core::LLVMDisposeMessage(c);

    let mut out_file = File::create("a.o").unwrap();
    emit(llvm_module, &mut out_file);

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

unsafe fn synthesise(
    statements: &Vec<Statement>,
) -> Result<(*mut llvm::LLVMContext, LLVMModuleRef), Error> {
    let ctx = llvm::core::LLVMContextCreate();
    let module = llvm::core::LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
    let builder = llvm::core::LLVMCreateBuilderInContext(ctx);

    let mut llvm_functions = HashMap::new();
    for statement in statements {
        if let &Statement::FnDefinition(ref name, ref params, ref expr) = statement {
            let function = Function::new(name, Some(params), None, expr);
            let llvm_function = function.synthesise(ctx, module, builder, &llvm_functions);
            llvm_functions.insert(name.clone(), llvm_function);
        }
    }

    let main_assigns = statements
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
        &Expression::Operand(Operand::I64(0)),
    );
    main_function.synthesise(ctx, module, builder, &llvm_functions);

    llvm::core::LLVMDisposeBuilder(builder);
    // FIXME: Error handling.
    return Ok((ctx, module));
}

unsafe fn var_assignment_codegen(
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    name: &Name,
    expr: &Expression,
    vars: &HashMap<Name, Var>,
    fns: &HashMap<Name, LLVMValueRef>,
) {
    let llvm_var = match vars.get(name).unwrap() {
        &Var::Stack(var) => var,
        &Var::Register(_) => unimplemented!(),
    };
    let value = ExpressionSynthesiser::synthesise(ctx, builder, expr, vars, fns);
    llvm::core::LLVMBuildStore(builder, value, llvm_var);
}

pub enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}
