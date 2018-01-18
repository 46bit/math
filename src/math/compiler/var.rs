use super::*;
use llvm;
use llvm::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

pub unsafe fn var_assignment_codegen(
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

pub unsafe fn synthesise_var_substitution(
    llvm_builder: LLVMBuilderRef,
    name: &Name,
    vars: &HashMap<Name, Var>,
) -> LLVMValueRef {
    match vars.get(name).unwrap() {
        &Var::Register(llvm_var) => llvm_var,
        &Var::Stack(llvm_var) => {
            let llvm_var_name = into_llvm_name(name.clone());
            llvm::core::LLVMBuildLoad(llvm_builder, llvm_var, llvm_var_name.as_ptr())
        }
    }
}
