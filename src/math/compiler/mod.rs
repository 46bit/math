mod function;

use super::*;
use self::function::*;
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

pub unsafe fn synthesise(statements: &Vec<Statement>) -> Result<(*mut llvm::LLVMContext, LLVMModuleRef), Error> {
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
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
    name: &Name,
    expr: &Expression,
    vars: &HashMap<Name, Var>,
    fns: &HashMap<Name, LLVMValueRef>,
) {
    let pointer = match vars.get(name).unwrap() {
        &Var::Register(_) => unimplemented!(),
        &Var::Stack(ref var) => *var,
    };
    let value = expression_codegen(ctx, module, builder, expr, vars, fns);
    llvm::core::LLVMBuildStore(builder, value, pointer);
}

unsafe fn expression_codegen(
    ctx: LLVMContextRef,
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
    expr: &Expression,
    vars: &HashMap<Name, Var>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> *mut llvm::LLVMValue {
    match expr {
        &Expression::Operand(ref operand) => {
            operand_codegen(ctx, module, builder, operand, vars, fns)
        }
        &Expression::Operation(operator, ref lhs_expr, ref rhs_expr) => operation_codegen(
            ctx,
            module,
            builder,
            operator,
            lhs_expr,
            rhs_expr,
            vars,
            fns,
        ),
    }
}

unsafe fn operand_codegen(
    ctx: LLVMContextRef,
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
    operand: &Operand,
    vars: &HashMap<Name, Var>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> *mut llvm::LLVMValue {
    let i64_type = llvm::core::LLVMInt64TypeInContext(ctx);
    match operand {
        &Operand::I64(n) => llvm::core::LLVMConstInt(i64_type, n as u64, 0),
        &Operand::Group(ref inner_expr) => {
            expression_codegen(ctx, module, builder, inner_expr, vars, fns)
        }
        &Operand::VarSubstitution(ref var_name) => match vars.get(var_name).unwrap() {
            &Var::Register(ref var) => *var,
            &Var::Stack(ref var) => {
                let name = into_llvm_name(var_name.clone());
                llvm::core::LLVMBuildLoad(builder, *var, name.as_ptr())
            }
        },
        &Operand::FnApplication(ref fn_name, ref arguments) => {
            let name = into_llvm_name(fn_name.clone());
            let function = fns.get(fn_name).unwrap();
            let mut argument_values: Vec<_> = arguments
                .iter()
                .map(|expr| expression_codegen(ctx, module, builder, expr, vars, fns))
                .collect();
            let argument_values_mut_slice = argument_values.as_mut_slice();
            llvm::core::LLVMBuildCall(
                builder,
                *function,
                argument_values_mut_slice.as_mut_ptr(),
                arguments.len() as u32,
                name.as_ptr(),
            )
        }
    }
}

unsafe fn operation_codegen(
    ctx: LLVMContextRef,
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
    operator: Operator,
    lhs_expr: &Expression,
    rhs_expr: &Expression,
    vars: &HashMap<Name, Var>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> *mut llvm::LLVMValue {
    let lhs = expression_codegen(ctx, module, builder, lhs_expr, vars, fns);
    let rhs = expression_codegen(ctx, module, builder, rhs_expr, vars, fns);
    let operation_name = llvm_name("tmp");
    match operator {
        Operator::Subtract => llvm::core::LLVMBuildSub(builder, lhs, rhs, operation_name.as_ptr()),
        Operator::Add => llvm::core::LLVMBuildAdd(builder, lhs, rhs, operation_name.as_ptr()),
        Operator::Divide => llvm::core::LLVMBuildSDiv(builder, lhs, rhs, operation_name.as_ptr()),
        Operator::Multiply => llvm::core::LLVMBuildMul(builder, lhs, rhs, operation_name.as_ptr()),
    }
}

enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}
