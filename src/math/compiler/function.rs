use super::*;
use llvm;
use llvm::prelude::*;
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
        external_functions: &HashMap<String, LLVMValueRef>,
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
    unsafe fn synthesise(
        &self,
        external_functions: &HashMap<String, LLVMValueRef>,
    ) -> LLVMValueRef {
        let llvm_fn = self.synthesise_fn();
        self.synthesise_entry(llvm_fn);
        let mut vars = self.synthesise_params(llvm_fn);
        self.synthesise_return(&mut vars, external_functions);
        return llvm_fn;
    }

    unsafe fn synthesise_fn(&self) -> LLVMValueRef {
        let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(self.llvm_ctx);

        let mut params_types: Vec<_> = self.function.params().map(|_| llvm_i64_type).collect();
        let llvm_fn_type = llvm::core::LLVMFunctionType(
            llvm_i64_type,
            params_types.as_mut_slice().as_mut_ptr(),
            params_types.len() as u32,
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

    unsafe fn synthesise_return(
        &self,
        vars: &HashMap<Name, Var>,
        external_functions: &HashMap<String, LLVMValueRef>,
    ) {
        llvm::core::LLVMBuildRet(
            self.llvm_builder,
            ExpressionSynthesiser::synthesise(
                self.llvm_ctx,
                self.llvm_builder,
                self.function.returns,
                &vars.iter().map(|(n, v)| (n.0.clone(), v.clone())).collect(),
                external_functions,
            ),
        );
    }
}

pub unsafe fn synthesise_fn_application(
    llvm_builder: LLVMBuilderRef,
    name: &Name,
    mut llvm_args: Vec<LLVMValueRef>,
    fns: &HashMap<String, LLVMValueRef>,
) -> LLVMValueRef {
    let llvm_function = fns.get(&name.0).unwrap();
    let llvm_fn_name = into_llvm_name(name.clone());
    let llvm_args_slice = llvm_args.as_mut_slice();
    llvm::core::LLVMBuildCall(
        llvm_builder,
        *llvm_function,
        llvm_args_slice.as_mut_ptr(),
        llvm_args_slice.len() as u32,
        llvm_fn_name.as_ptr(),
    )
}
