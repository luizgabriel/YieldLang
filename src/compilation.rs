#![allow(dead_code)]
use inkwell::{builder::BuilderError, module::Linkage, values::*, *};
use thiserror::Error;

use crate::ast::Expr;

pub struct LLVMCodeGen<'a> {
    context: &'a context::Context,
    module: module::Module<'a>,
    builder: builder::Builder<'a>,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("BuilderError: {0}")]
    BuilderError(#[from] BuilderError),

    #[error("Invalid main function")]
    InvalidMainFunction,
}

impl<'a> LLVMCodeGen<'a> {
    pub fn new(context: &'a context::Context, module_name: &str) -> LLVMCodeGen<'a> {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        LLVMCodeGen {
            context,
            module,
            builder,
        }
    }

    pub fn print_to_file(&self, file_name: &str) -> Result<(), String> {
        self.module
            .print_to_file(file_name)
            .map_err(|e| e.to_string())
    }

    fn generate(&self, expr: Expr) -> Result<BasicValueEnum<'a>, CompileError> {
        match expr {
            Expr::Integer(value) => {
                let i32_type = self.context.i32_type();
                Ok(BasicValueEnum::IntValue(
                    i32_type.const_int(value as u64, false),
                ))
            }
            Expr::String(value) => {
                let string_ptr = self.builder.build_global_string_ptr(&value, "")?;
                Ok(BasicValueEnum::PointerValue(string_ptr.as_pointer_value()))
            }
            _ => unimplemented!(),
        }
    }

    fn define_libc_functions(&self) {
        let i8_type = self.context.i8_type();
        let i32_type = self.context.i32_type();

        // Define "puts" function
        let puts_type =
            i32_type.fn_type(&[i8_type.ptr_type(AddressSpace::default()).into()], false);
        let function = self
            .module
            .add_function("puts", puts_type, Some(Linkage::External));
        assert!(function.verify(false));
    }

    pub fn compile(&self, expr: Expr) -> Result<(), CompileError> {
        self.define_libc_functions();

        let i32_type = self.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn_value = self.module.add_function("main", main_fn_type, None);

        let entry_block = self.context.append_basic_block(main_fn_value, "entry");
        self.builder.position_at_end(entry_block);

        let value = self.generate(expr)?;

        match value {
            BasicValueEnum::IntValue(value) => {
                self.builder.build_return(Some(&value))?;
            }

            BasicValueEnum::PointerValue(value) => {
                let puts_fn = self.module.get_function("puts").unwrap();
                self.builder.build_call(puts_fn, &[value.into()], "")?;
                self.builder
                    .build_return(Some(&i32_type.const_int(0, false)))?;
            }

            _ => {
                self.builder
                    .build_return(Some(&i32_type.const_int(0, false)))?;
            }
        }

        if !main_fn_value.verify(false) {
            return Err(CompileError::InvalidMainFunction);
        }

        Ok(())
    }
}
