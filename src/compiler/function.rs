use super::*;
use llvm::prelude::*;
use llvm::core::*;
use std::iter;
use std::collections::HashMap;

pub unsafe fn synthesise_function(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    name: &Name,
    params: &Vec<Name>,
    returns: &Expression,
    functions: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    FunctionSynthesiser {
        ctx: ctx,
        module: module,
        builder: builder,
        functions: functions,
    }.synthesise(name, &params, returns)
}

struct FunctionSynthesiser<'a> {
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    functions: &'a HashMap<Name, LLVMValueRef>,
}

impl<'a> FunctionSynthesiser<'a> {
    unsafe fn synthesise(
        &self,
        name: &Name,
        params: &Vec<Name>,
        returns: &Expression,
    ) -> LLVMValueRef {
        let (function, mut vars) = self.synthesise_fn(name, params);
        self.synthesise_entry(function);
        self.synthesise_return(function, returns, &mut vars);
        function
    }

    unsafe fn synthesise_fn(
        &self,
        name: &Name,
        params: &Vec<Name>,
    ) -> (LLVMValueRef, HashMap<Name, LLVMValueRef>) {
        let i64_type = LLVMInt64TypeInContext(self.ctx);
        let params = params.iter().cloned().zip(iter::repeat(i64_type)).collect();
        let name = into_llvm_name(name.clone());
        function_definition(self.module, name, params, i64_type)
    }

    unsafe fn synthesise_entry(&self, function: LLVMValueRef) {
        function_entry(self.ctx, self.builder, llvm_name("entry"), function);
    }

    unsafe fn synthesise_return(
        &self,
        function: LLVMValueRef,
        returns: &Expression,
        vars: &HashMap<Name, LLVMValueRef>,
    ) {
        let value = synthesise_expression(
            self.ctx,
            self.module,
            self.builder,
            function,
            returns,
            vars,
            self.functions,
        );
        LLVMBuildRet(self.builder, value);
    }
}

pub unsafe fn synthesise_fn_application(
    builder: LLVMBuilderRef,
    name: &Name,
    mut args: Vec<LLVMValueRef>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let function = assert_not_nil(*fns.get(&name).unwrap());
    let fn_name = into_llvm_name(name.clone());
    let args_slice = args.as_mut_slice();
    assert_not_nil(LLVMBuildCall(
        builder,
        function,
        args_slice.as_mut_ptr(),
        args_slice.len() as u32,
        fn_name.as_ptr(),
    ))
}
