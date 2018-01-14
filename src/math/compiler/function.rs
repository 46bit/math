use super::*;
use llvm;
use llvm::prelude::*;
use std::collections::HashMap;

pub struct Function<'a> {
    pub name: &'a Name,
    params: Option<&'a Vec<Name>>,
    assigns: Option<HashMap<&'a Name, &'a Expression>>,
    pub returns: &'a Expression,
}

impl<'a> Function<'a> {
    pub fn new(
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

    pub fn params_count(&self) -> u32 {
        match self.params {
            Some(v) => v.len() as u32,
            None => 0,
        }
    }

    pub fn params(&'a self) -> impl Iterator<Item = &'a Name> + 'a {
        self.params.iter().flat_map(|v| v.iter())
    }

    pub fn assigns(&self) -> impl Iterator<Item = (&&Name, &&Expression)> {
        self.assigns.iter().flat_map(|v| v.iter())
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