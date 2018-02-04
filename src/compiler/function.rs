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
        let (llvm_fn, mut vars) = self.synthesise_fn();
        self.synthesise_entry(llvm_fn);
        self.synthesise_return(&mut vars, external_functions);
        return llvm_fn;
    }

    unsafe fn synthesise_fn(&self) -> (LLVMValueRef, HashMap<Name, LLVMValueRef>) {
        let i64_type = LLVMInt64TypeInContext(self.llvm_ctx);
        let name = into_llvm_name(self.function.name.clone());
        let params = self.function
            .params()
            .cloned()
            .zip(iter::repeat(i64_type))
            .collect();
        llvm_function_definition(self.llvm_module, name, params, i64_type)
    }

    unsafe fn synthesise_entry(&self, llvm_fn: LLVMValueRef) {
        llvm_function_entry(
            self.llvm_ctx,
            self.llvm_builder,
            llvm_name("entry"),
            llvm_fn,
        );
    }

    unsafe fn synthesise_return(
        &self,
        vars: &HashMap<Name, LLVMValueRef>,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) {
        LLVMBuildRet(
            self.llvm_builder,
            ExpressionSynthesiser::synthesise(
                self.llvm_ctx,
                self.llvm_builder,
                self.function.returns,
                &vars.iter().map(|(n, v)| (n.clone(), v.clone())).collect(),
                external_functions,
            ),
        );
    }
}

pub unsafe fn synthesise_fn_application(
    llvm_builder: LLVMBuilderRef,
    name: &Name,
    mut llvm_args: Vec<LLVMValueRef>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let llvm_function = fns.get(&name).unwrap();
    let llvm_fn_name = into_llvm_name(name.clone());
    let llvm_args_slice = llvm_args.as_mut_slice();
    LLVMBuildCall(
        llvm_builder,
        *llvm_function,
        llvm_args_slice.as_mut_ptr(),
        llvm_args_slice.len() as u32,
        llvm_fn_name.as_ptr(),
    )
}
