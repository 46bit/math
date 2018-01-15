mod function;
mod expression;

use super::*;
use self::function::*;
use self::expression::*;
use llvm;
use llvm::prelude::*;
use std::ffi::{CStr, CString};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {}

pub unsafe fn compile(statements: &Statements) -> Result<String, Error> {
    let (ctx, module) = synthesise(&statements.0)?;

    let c = llvm::core::LLVMPrintModuleToString(module);
    let llvm_ir = CStr::from_ptr(c).to_string_lossy().into_owned();
    llvm::core::LLVMDisposeMessage(c);

    llvm::core::LLVMDisposeModule(module);
    llvm::core::LLVMContextDispose(ctx);
    // FIXME: Error handling.
    return Ok(llvm_ir);
}

pub unsafe fn synthesise(
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
