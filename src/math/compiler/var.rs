use super::*;
use llvm;
use llvm::prelude::*;

#[derive(Debug, Clone)]
pub enum Var {
    Register(LLVMValueRef),
    Stack(LLVMValueRef),
}

impl Var {
    pub unsafe fn synthesise_allocation(
        llvm_builder: LLVMBuilderRef,
        name: Name,
        llvm_type: LLVMTypeRef,
    ) -> Var {
        let llvm_name = into_llvm_name(name);
        Var::Stack(llvm::core::LLVMBuildAlloca(
            llvm_builder,
            llvm_type,
            llvm_name.as_ptr(),
        ))
    }

    pub unsafe fn synthesise_assignment(
        &self,
        llvm_builder: LLVMBuilderRef,
        llvm_value: LLVMValueRef,
    ) {
        match *self {
            Var::Stack(llvm_var) => {
                llvm::core::LLVMBuildStore(llvm_builder, llvm_value, llvm_var);
            }
            Var::Register(_) => unimplemented!(),
        }
    }

    pub unsafe fn synthesise_pointer(
        &self,
        llvm_builder: LLVMBuilderRef,
        name: &Name,
    ) -> LLVMValueRef {
        match *self {
            Var::Stack(llvm_var) => {
                let llvm_ptr_name = llvm_name(&format!("ptr_{}", name.0));
                llvm_var
                // llvm::core::LLVMBuildPointerCast(
                //     llvm_builder,
                //     llvm_var,
                //     llvm::core::LLVMTypeOf(llvm_var),
                //     llvm_ptr_name.as_ptr(),
                // )
            }
            Var::Register(_) => unimplemented!(),
        }
    }

    pub unsafe fn synthesise_substitution(
        &self,
        llvm_builder: LLVMBuilderRef,
        name: &Name,
    ) -> LLVMValueRef {
        match *self {
            Var::Register(llvm_var) => llvm_var,
            Var::Stack(llvm_var) => {
                let llvm_var_name = into_llvm_name(name.clone());
                llvm::core::LLVMBuildLoad(llvm_builder, llvm_var, llvm_var_name.as_ptr())
            }
        }
    }
}
