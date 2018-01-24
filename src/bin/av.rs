extern crate llvm_sys as llvm;
#[macro_use]
extern crate nom;

use llvm::prelude::*;
use std::ptr;
use std::ffi::{CStr, CString};

fn main() {
    unsafe {
        let ctx = llvm::core::LLVMContextCreate();
        let module = llvm::core::LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
        let builder = llvm::core::LLVMCreateBuilderInContext(ctx);

        let string_type = llvm::core::LLVMPointerType(llvm::core::LLVMInt8Type(), 0);
        let i32_type = llvm::core::LLVMInt32TypeInContext(ctx);
        let i64_type = llvm::core::LLVMInt64TypeInContext(ctx);
        let i64_ptr_type = llvm::core::LLVMPointerType(i64_type, 0);

        let params_types = &mut [string_type, string_type, i64_ptr_type];
        let sscanf_type = llvm::core::LLVMFunctionType(i32_type, params_types.as_mut_ptr(), 3, 1);
        let sscanf_name = llvm_name("sscanf");
        let sscanf = llvm::core::LLVMAddFunction(module, sscanf_name.as_ptr(), sscanf_type);

        let argc_name = llvm_name("argc");
        let argc_type = i64_type;
        let argv_name = llvm_name("argv");
        let argv_type = llvm::core::LLVMPointerType(
            llvm::core::LLVMPointerType(llvm::core::LLVMInt8Type(), 0),
            0,
        );
        let param_types = &mut [argc_type, argv_type];
        let return_type = i64_type;
        let main_name = llvm_name("main");
        let main_type = llvm::core::LLVMFunctionType(
            return_type,
            param_types.as_mut_ptr(),
            param_types.len() as u32,
            0,
        );
        let main = llvm::core::LLVMAddFunction(module, main_name.as_ptr(), main_type);
        let argc = llvm::core::LLVMGetParam(main, 0);
        let argv = llvm::core::LLVMGetParam(main, 1);
        llvm::core::LLVMSetValueName(argc, argc_name.as_ptr());
        llvm::core::LLVMSetValueName(argv, argv_name.as_ptr());
        let main_entry_name = llvm_name("entry");
        let main_entry =
            llvm::core::LLVMAppendBasicBlockInContext(ctx, main, main_entry_name.as_ptr());
        llvm::core::LLVMPositionBuilderAtEnd(builder, main_entry);

        let sscanf_tmpl_name = llvm_name("sscanf_template");
        let sscanf_tmpl_string = llvm_name("%d");
        let sscanf_tmpl = llvm::core::LLVMBuildGlobalStringPtr(
            builder,
            sscanf_tmpl_string.as_ptr(),
            sscanf_tmpl_name.as_ptr(),
        );
        //let argv_el = llvm::core::LLVMBuildExtractValue(builder, argv, 1, argv_el_name.as_ptr());
        //let argv_el = llvm::core::LLVMBuildLoad(builder, argv, argv_el_name.as_ptr());
        let indices_name = llvm_name("indices");
        let indices = &mut [llvm::core::LLVMConstInt(i64_type, 1 as u64, 0)];
        let argv_els = llvm::core::LLVMBuildGEP(
            builder,
            argv,
            indices.as_mut_ptr(),
            1,
            indices_name.as_ptr(),
        );
        let argv_el_name = llvm_name("argv_el");
        let argv_el = llvm::core::LLVMBuildLoad(builder, argv_els, argv_el_name.as_ptr());
        let input_name = llvm_name("input");
        let input = llvm::core::LLVMBuildAlloca(builder, i64_type, input_name.as_ptr());
        let args_slice = &mut [argv_el, sscanf_tmpl, input];
        let call_name = llvm_name("tmp");
        llvm::core::LLVMBuildCall(
            builder,
            sscanf,
            args_slice.as_mut_ptr(),
            args_slice.len() as u32,
            call_name.as_ptr(),
        );
        // FIXME: Pseudocode for parsing argv into inputs
        // assert(argc >= params.len())
        // sscanf(argv, 0..params.len().map(|_| "%d").collect::<Vec<_>>().join(" "), &p0, &p1, ...)

        let output_name = llvm_name("output");
        let output = llvm::core::LLVMBuildLoad(builder, input, output_name.as_ptr());
        llvm::core::LLVMBuildRet(builder, output);

        llvm::core::LLVMDisposeBuilder(builder);

        let c = llvm::core::LLVMPrintModuleToString(module);
        let ir = CStr::from_ptr(c).to_string_lossy().into_owned();
        llvm::core::LLVMDisposeMessage(c);

        llvm::core::LLVMDisposeModule(module);
        llvm::core::LLVMContextDispose(ctx);

        println!("{}", ir);
    }
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}
