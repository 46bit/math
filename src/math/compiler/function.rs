use super::*;
use llvm;
use llvm::prelude::*;
use std::collections::HashMap;

pub struct Function<'a> {
    pub name: &'a Name,
    params: Option<&'a Vec<Name>>,
    assigns: Option<HashMap<&'a Name, &'a Expression>>,
    prints: Option<&'a Vec<Name>>,
    pub returns: &'a Expression,
}

impl<'a> Function<'a> {
    pub fn new(
        name: &'a Name,
        params: Option<&'a Vec<Name>>,
        assigns: Option<HashMap<&'a Name, &'a Expression>>,
        prints: Option<&'a Vec<Name>>,
        returns: &'a Expression,
    ) -> Function<'a> {
        Function {
            name: name,
            params: params,
            assigns: assigns,
            prints: prints,
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

    pub fn prints(&'a self) -> impl Iterator<Item = &'a Name> + 'a {
        self.prints.iter().flat_map(|v| v.iter())
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
        self.synthesise_entry(llvm_fn);
        let mut vars = self.synthesise_params(llvm_fn, external_functions);
        self.synthesise_assignments(&mut vars, external_functions);
        self.synthesise_prints(&mut vars, external_functions);
        self.synthesise_return(&mut vars, external_functions);
        return llvm_fn;
    }

    unsafe fn synthesise_fn(&self) -> LLVMValueRef {
        let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(self.llvm_ctx);

        let mut params_types: Vec<_>;
        if self.function.name == &Name("main".to_string()) {
            let llvm_argv_type = llvm::core::LLVMPointerType(
                llvm::core::LLVMPointerType(llvm::core::LLVMInt8Type(), 0),
                0,
            );
            params_types = vec![llvm_i64_type, llvm_argv_type];
        } else {
            params_types = self.function.params().map(|_| llvm_i64_type).collect();
        }
        let llvm_fn_type = llvm::core::LLVMFunctionType(
            llvm_i64_type,
            params_types.as_mut_slice().as_mut_ptr(),
            params_types.len() as u32,
            0,
        );

        let llvm_fn_name = into_llvm_name(self.function.name.clone());
        llvm::core::LLVMAddFunction(self.llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type)
    }

    unsafe fn synthesise_params(
        &self,
        llvm_fn: LLVMValueRef,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) -> HashMap<Name, Var> {
        if self.function.name == &Name("main".to_string()) {
            let llvm_argc_param = llvm::core::LLVMGetParam(llvm_fn, 0);
            let llvm_argc_name = llvm_name("argc");
            llvm::core::LLVMSetValueName(llvm_argc_param, llvm_argc_name.as_ptr());

            let llvm_argv_param = llvm::core::LLVMGetParam(llvm_fn, 1);
            let llvm_argv_name = llvm_name("argv");
            llvm::core::LLVMSetValueName(llvm_argv_param, llvm_argv_name.as_ptr());

            // FIXME: assert(argc >= params.len())

            let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(self.llvm_ctx);

            let mut vars = HashMap::new();
            for (i, param_name) in self.function.params().enumerate() {
                let arg_name = llvm_name("arg");
                let arg_idx = &mut [llvm::core::LLVMConstInt(llvm_i64_type, 1 + i as u64, 0)];
                let arg_ptr = llvm::core::LLVMBuildGEP(
                    self.llvm_builder,
                    llvm_argv_param,
                    arg_idx.as_mut_ptr(),
                    1,
                    arg_name.as_ptr(),
                );
                let arg = llvm::core::LLVMBuildLoad(self.llvm_builder, arg_ptr, arg_name.as_ptr());

                let tmpl_name = llvm_name("tmpl");
                let tmpl_string = llvm_name("%lld");
                let tmpl = llvm::core::LLVMBuildGlobalStringPtr(
                    self.llvm_builder,
                    tmpl_string.as_ptr(),
                    tmpl_name.as_ptr(),
                );

                let input = Var::synthesise_allocation(
                    self.llvm_builder,
                    param_name.clone(),
                    llvm_i64_type,
                );

                let call_name = llvm_name("call");
                let args = &mut [arg, tmpl, input.synthesise_pointer()];
                llvm::core::LLVMBuildCall(
                    self.llvm_builder,
                    external_functions[&Name("sscanf".to_string())],
                    args.as_mut_ptr(),
                    args.len() as u32,
                    call_name.as_ptr(),
                );

                vars.insert(param_name.clone(), input);
            }

            vars.into_iter().collect()
        } else {
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
            let var_name = var_name.clone().clone();
            vars.insert(
                var_name.clone(),
                Var::synthesise_allocation(self.llvm_builder, var_name, llvm_i64_type),
            );
        }

        for (ref var_name, ref var_expression) in self.function.assigns() {
            vars[var_name].synthesise_assignment(
                self.llvm_builder,
                ExpressionSynthesiser::synthesise(
                    self.llvm_ctx,
                    self.llvm_builder,
                    var_expression,
                    &vars,
                    &external_functions,
                ),
            );
        }
    }

    unsafe fn synthesise_prints(
        &self,
        vars: &mut HashMap<Name, Var>,
        external_functions: &HashMap<Name, LLVMValueRef>,
    ) {
        for var_name in self.function.prints() {
            let printf_template_name = llvm_name("printf_template");
            let printf_template = llvm_name("%lld\n");
            let llvm_printf_template = llvm::core::LLVMBuildGlobalStringPtr(
                self.llvm_builder,
                printf_template.as_ptr(),
                printf_template_name.as_ptr(),
            );

            let llvm_var = vars[var_name].synthesise_substitution(self.llvm_builder, var_name);
            synthesise_fn_application(
                self.llvm_builder,
                &Name::new("printf"),
                vec![llvm_printf_template, llvm_var],
                external_functions,
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
            ExpressionSynthesiser::synthesise(
                self.llvm_ctx,
                self.llvm_builder,
                self.function.returns,
                &mut vars,
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
    let llvm_function = fns.get(name).unwrap();
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
