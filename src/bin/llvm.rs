#![feature(box_syntax)]

extern crate advent;
extern crate llvm_sys as llvm;

use std::ptr;
use std::ffi::{CStr, CString};
use std::io::{stdin, Read};
use std::collections::HashMap;
use llvm::prelude::*;
use advent::math::*;

enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

fn main() {
    let mut input = String::new();
    let stdin = stdin();
    let mut stdin_lock = stdin.lock();
    stdin_lock.read_to_string(&mut input).unwrap();
    let statements = parser::parse(input.as_str().as_bytes()).unwrap();

    unsafe {
        codegen(statements.0);
    }
}

unsafe fn codegen(statements: Vec<Statement>) {
    let ctx = llvm::core::LLVMContextCreate();
    let module = llvm::core::LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
    let builder = llvm::core::LLVMCreateBuilderInContext(ctx);

    let function_name = llvm_name("main");
    let i64_type = llvm::core::LLVMInt64TypeInContext(ctx);
    let function_type = llvm::core::LLVMFunctionType(i64_type, ptr::null_mut(), 0, 0);
    let main_function = llvm::core::LLVMAddFunction(module, function_name.as_ptr(), function_type);

    let entry_name = llvm_name("entry");
    let bb = llvm::core::LLVMAppendBasicBlockInContext(ctx, main_function, entry_name.as_ptr());

    let mut vars = HashMap::new();
    let mut fns = HashMap::new();
    for statement in &statements {
        if let &Statement::FnDefinition(ref name, ref params, ref expr) = statement {
            fn_definition_codegen(ctx, module, builder, name, params, expr, &mut fns);
        }
    }

    llvm::core::LLVMPositionBuilderAtEnd(builder, bb);

    for statement in &statements {
        if let &Statement::VarAssignment(ref name, ..) = statement {
            let var_name = into_llvm_name(name.clone());
            let pointer = llvm::core::LLVMBuildAlloca(builder, i64_type, var_name.as_ptr());
            vars.insert(name.clone(), Var::Stack(pointer));
        }
    }
    for statement in &statements {
        if let &Statement::VarAssignment(ref name, ref expr) = statement {
            var_assignment_codegen(ctx, module, builder, name, expr, &vars, &fns);
        }
    }

    llvm::core::LLVMBuildRet(builder, llvm::core::LLVMConstInt(i64_type, 0, 0));

    let c = llvm::core::LLVMPrintModuleToString(module);
    let llvm_ir_cstring = CStr::from_ptr(c).to_string_lossy();
    println!("{}", llvm_ir_cstring);
    llvm::core::LLVMDisposeMessage(c);

    llvm::core::LLVMDisposeBuilder(builder);
    llvm::core::LLVMDisposeModule(module);
    llvm::core::LLVMContextDispose(ctx);
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
        &Var::Stack(ref var) => *var
    };
    let value = expression_codegen(ctx, module, builder, expr, vars, fns);
    llvm::core::LLVMBuildStore(builder, value, pointer);
}

unsafe fn fn_definition_codegen(
    ctx: LLVMContextRef,
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
    name: &Name,
    params: &Vec<Name>,
    expr: &Expression,
    fns: &mut HashMap<Name, LLVMValueRef>,
) {
    let i64_type = llvm::core::LLVMInt64TypeInContext(ctx);

    let function_name = into_llvm_name(name.clone());

    let mut params_names: Vec<*mut llvm::LLVMType> = params.iter().map(|_| i64_type).collect();
    let params_names2 = params_names.as_mut_slice();
    let function_type =
        llvm::core::LLVMFunctionType(i64_type, params_names2.as_mut_ptr(), params.len() as u32, 0);
    let function = llvm::core::LLVMAddFunction(module, function_name.as_ptr(), function_type);

    let mut arguments = HashMap::new();
    for (i, param_name) in params.iter().enumerate() {
        let param = llvm::core::LLVMGetParam(function, i as u32);
        let name = into_llvm_name(param_name.clone());
        llvm::core::LLVMSetValueName(param, name.as_ptr());
        arguments.insert(param_name.clone(), Var::Register(param));
    }

    let entry_name = llvm_name("entry");
    let bb = llvm::core::LLVMAppendBasicBlockInContext(ctx, function, entry_name.as_ptr());
    llvm::core::LLVMPositionBuilderAtEnd(builder, bb);

    llvm::core::LLVMBuildRet(
        builder,
        expression_codegen(ctx, module, builder, expr, &mut arguments, fns),
    );

    fns.insert(name.clone(), function);
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
        &Operand::VarSubstitution(ref var_name) => {
            match vars.get(var_name).unwrap() {
                &Var::Register(ref var) => *var,
                &Var::Stack(ref var) => {
                    let name = into_llvm_name(var_name.clone());
                    llvm::core::LLVMBuildLoad(builder, *var, name.as_ptr())
                }
            }
        }
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

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}

//unsafe fn fn_definition_codegen(name: Name, params: Vec<Name>, coefficient: i64) {

//unsafe fn fn_definition_codegen(name: Name, params: Vec<Name>, expr: Expression) {
