use super::*;
use llvm::prelude::*;
use llvm::core::*;
use std::iter;
use std::collections::HashMap;

pub struct Function<'a> {
    pub name: &'a Name,
    params: Option<&'a Vec<Name>>,
    pub returns: &'a Expression,
}

impl<'a> Function<'a> {
    pub fn new(
        name: &'a Name,
        params: Option<&'a Vec<Name>>,
        returns: &'a Expression,
    ) -> Function<'a> {
        Function {
            name: name,
            params: params,
            returns: returns,
        }
    }

    pub fn params(&'a self) -> impl Iterator<Item = &'a Name> + 'a {
        self.params.iter().flat_map(|v| v.iter())
    }

    pub unsafe fn synthesise(
        &self,
        ctx: LLVMContextRef,
        module: *mut llvm::LLVMModule,
        builder: LLVMBuilderRef,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) -> LLVMValueRef {
        FunctionSynthesiser {
            function: &self,
            ctx: ctx,
            module: module,
            builder: builder,
        }.synthesise(external_functions)
    }
}

struct FunctionSynthesiser<'a> {
    function: &'a Function<'a>,
    ctx: LLVMContextRef,
    module: *mut llvm::LLVMModule,
    builder: LLVMBuilderRef,
}

impl<'a> FunctionSynthesiser<'a> {
    unsafe fn synthesise(&self, external_functions: &HashMap<Name, LLVMValueRef>) -> LLVMValueRef {
        let (function, mut vars) = self.synthesise_fn();
        self.synthesise_entry(function);
        self.synthesise_return(&mut vars, external_functions);
        function
    }

    unsafe fn synthesise_fn(&self) -> (LLVMValueRef, HashMap<Name, LLVMValueRef>) {
        let i64_type = LLVMInt64TypeInContext(self.ctx);
        let name = into_llvm_name(self.function.name.clone());
        let params = self.function
            .params()
            .cloned()
            .zip(iter::repeat(i64_type))
            .collect();
        function_definition(self.module, name, params, i64_type)
    }

    unsafe fn synthesise_entry(&self, function: LLVMValueRef) {
        function_entry(self.ctx, self.builder, llvm_name("entry"), function);
    }

    unsafe fn synthesise_return(
        &self,
        vars: &HashMap<Name, LLVMValueRef>,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) {
        LLVMBuildRet(
            self.builder,
            ExpressionSynthesiser::synthesise(
                self.ctx,
                self.module,
                self.builder,
                self.function.returns,
                &vars.iter().map(|(n, v)| (n.clone(), v.clone())).collect(),
                external_functions,
            ),
        );
    }
}

pub unsafe fn synthesise_fn_application(
    builder: LLVMBuilderRef,
    name: &Name,
    mut args: Vec<LLVMValueRef>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let function = fns.get(&name).unwrap();
    let fn_name = into_llvm_name(name.clone());
    let args_slice = args.as_mut_slice();
    LLVMBuildCall(
        builder,
        *function,
        args_slice.as_mut_ptr(),
        args_slice.len() as u32,
        fn_name.as_ptr(),
    )
}
