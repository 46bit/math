#![feature(box_syntax)]
#![feature(conservative_impl_trait)]

extern crate advent;
extern crate llvm_sys as llvm;

use std::ffi::{CStr, CString};
use std::io::{stdin, Read};
use std::collections::HashMap;
use llvm::prelude::*;
use advent::math::*;

enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

struct Function<'a> {
    name: &'a Name,
    params: Option<&'a Vec<Name>>,
    assigns: Option<HashMap<&'a Name, &'a Expression>>,
    returns: &'a Expression,
}

impl<'a> Function<'a> {
    fn new(
        name: &'a Name,
        params: Option<&'a Vec<Name>>,
        assigns: Option<HashMap<&'a Name, &'a Expression>>,
        returns: &'a Expression,
    ) -> Function<'a> {
        Function {
            name: name,
            params: params,
            assigns: assigns,
            returns: returns,
        }
    }

    fn params_count(&self) -> u32 {
        match self.params {
            Some(v) => v.len() as u32,
            None => 0,
        }
    }

    fn params(&'a self) -> impl Iterator<Item = &'a Name> + 'a {
        self.params.iter().flat_map(|v| v.iter())
    }

    fn assigns(&self) -> impl Iterator<Item = (&&Name, &&Expression)> {
        self.assigns.iter().flat_map(|v| v.iter())
    }

    unsafe fn synthesise(
        &self,
        llvm_ctx: LLVMContextRef,
        llvm_module: *mut llvm::LLVMModule,
        llvm_builder: LLVMBuilderRef,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) -> LLVMValueRef {
        FunctionSynthesiser {
            function: &self,
            llvm_ctx: llvm_ctx,
            llvm_module: llvm_module,
            llvm_builder: llvm_builder,
        }.synthesise(external_functions)
    }
}

struct FunctionSynthesiser<'a> {
    function: &'a Function<'a>,
    llvm_ctx: LLVMContextRef,
    llvm_module: *mut llvm::LLVMModule,
    llvm_builder: LLVMBuilderRef,
}

impl<'a> FunctionSynthesiser<'a> {
    unsafe fn synthesise(&self, external_functions: &HashMap<Name, LLVMValueRef>) -> LLVMValueRef {
        let llvm_fn = self.synthesise_fn();
        let mut vars = self.synthesise_params(llvm_fn);
        self.synthesise_entry(llvm_fn);
        self.synthesise_assignments(&mut vars, external_functions);
        self.synthesise_return(&mut vars, external_functions);
        return llvm_fn;
    }

    unsafe fn synthesise_fn(&self) -> LLVMValueRef {
        let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(self.llvm_ctx);
        let mut params_types: Vec<_> = self.function.params().map(|_| llvm_i64_type).collect();
        let llvm_fn_type = llvm::core::LLVMFunctionType(
            llvm_i64_type,
            params_types.as_mut_slice().as_mut_ptr(),
            self.function.params_count(),
            0,
        );

        let llvm_fn_name = into_llvm_name(self.function.name.clone());
        llvm::core::LLVMAddFunction(self.llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type)
    }

    unsafe fn synthesise_params(&self, llvm_fn: LLVMValueRef) -> HashMap<Name, Var> {
        self.function
            .params()
            .enumerate()
            .map(|(param_index, param_name)| {
                let llvm_param = llvm::core::LLVMGetParam(llvm_fn, param_index as u32);
                let llvm_param_name = into_llvm_name(param_name.clone());
                llvm::core::LLVMSetValueName(llvm_param, llvm_param_name.as_ptr());
                (param_name.clone(), Var::Register(llvm_param))
            })
            .collect()
    }

    unsafe fn synthesise_entry(&self, llvm_fn: LLVMValueRef) {
        let llvm_fn_entry_name = llvm_name("entry");
        let llvm_fn_entry_block = llvm::core::LLVMAppendBasicBlockInContext(
            self.llvm_ctx,
            llvm_fn,
            llvm_fn_entry_name.as_ptr(),
        );
        llvm::core::LLVMPositionBuilderAtEnd(self.llvm_builder, llvm_fn_entry_block);
    }

    unsafe fn synthesise_assignments(
        &self,
        vars: &mut HashMap<Name, Var>,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) {
        let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(self.llvm_ctx);

        for (var_name, _) in self.function.assigns() {
            let llvm_var_name = into_llvm_name(var_name.clone().clone());
            let llvm_var_pointer = llvm::core::LLVMBuildAlloca(
                self.llvm_builder,
                llvm_i64_type,
                llvm_var_name.as_ptr(),
            );
            vars.insert(var_name.clone().clone(), Var::Stack(llvm_var_pointer));
        }

        for (ref var_name, ref var_expression) in self.function.assigns() {
            var_assignment_codegen(
                self.llvm_ctx,
                self.llvm_module,
                self.llvm_builder,
                var_name,
                var_expression,
                &vars,
                &external_functions,
            );
        }
    }

    unsafe fn synthesise_return(
        &self,
        mut vars: &mut HashMap<Name, Var>,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) {
        llvm::core::LLVMBuildRet(
            self.llvm_builder,
            expression_codegen(
                self.llvm_ctx,
                self.llvm_module,
                self.llvm_builder,
                self.function.returns,
                &mut vars,
                external_functions,
            ),
        );
    }
}

fn main() {
    let mut input = String::new();
    let stdin = stdin();
    let mut stdin_lock = stdin.lock();
    stdin_lock.read_to_string(&mut input).unwrap();
    let statements = parser::parse(input.as_str().as_bytes()).unwrap();

    unsafe {
        codegen(&statements.0);
    }
}

unsafe fn codegen(statements: &Vec<Statement>) {
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

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}

//unsafe fn fn_definition_codegen(name: Name, params: Vec<Name>, coefficient: i64) {

//unsafe fn fn_definition_codegen(name: Name, params: Vec<Name>, expr: Expression) {
