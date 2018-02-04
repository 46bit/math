use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::*;

pub unsafe fn llvm_function_definition(
    module: LLVMModuleRef,
    name: CString,
    params: Vec<(Name, LLVMTypeRef)>,
    return_type: LLVMTypeRef,
) -> (LLVMValueRef, HashMap<Name, LLVMValueRef>) {
    let mut param_types: Vec<_> = params.iter().map(|&(_, param_type)| param_type).collect();
    let function = LLVMAddFunction(
        module,
        name.as_ptr(),
        LLVMFunctionType(
            return_type,
            param_types.as_mut_slice().as_mut_ptr(),
            param_types.len() as u32,
            0,
        ),
    );
    let mut param_values = HashMap::new();
    for (i, (param_name, _)) in params.into_iter().enumerate() {
        let param = LLVMGetParam(function, i as u32);
        let llvm_param_name = param_name.clone().cstring();
        LLVMSetValueName(param, llvm_param_name.as_ptr());
        param_values.insert(param_name.clone(), param);
    }
    (function, param_values)
}

pub unsafe fn llvm_function_entry(
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    name: CString,
    function: LLVMValueRef,
) {
    let block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    LLVMPositionBuilderAtEnd(builder, block);
}

pub unsafe fn llvm_function_return(builder: LLVMBuilderRef, value: LLVMValueRef) {
    LLVMBuildRet(builder, value);
}
