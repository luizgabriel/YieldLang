#![allow(dead_code)]
use inkwell::{builder::BuilderError, module::Linkage, types::AnyTypeEnum, values::*, *};
use thiserror::Error;

use crate::ast::{Expr, LiteralValue};

pub struct LLVMCodeGen<'a> {
    context: &'a context::Context,
    module: module::Module<'a>,
    builder: builder::Builder<'a>,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("BuilderError: {0}")]
    BuilderError(#[from] BuilderError),

    #[error("BasicValueRequired: {0}")]
    BasicValueRequired(String),

    #[error("Function does not exist: {0}")]
    UndefinedFunction(String),

    #[error("Literal values are not callable")]
    NotAFunction(),

    #[error("Empty block")]
    EmptyBlock,

    #[error("Invalid main function")]
    InvalidMainFunction,
}

fn generate_bool<'a>(context: &'a context::Context, value: bool) -> AnyValueEnum<'a> {
    let bool_type = context.bool_type();
    AnyValueEnum::IntValue(bool_type.const_int(value.into(), false))
}

fn generate_i32<'a>(context: &'a context::Context, value: i32) -> AnyValueEnum<'a> {
    let i32_type = context.i32_type();
    AnyValueEnum::IntValue(i32_type.const_int(value as u64, false))
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

    fn generate(&self, expr: Expr) -> Result<AnyValueEnum<'a>, CompileError> {
        match expr {
            Expr::Literal(literal) => match literal {
                LiteralValue::Boolean(value) => Ok(generate_bool(self.context, value)),
                LiteralValue::Integer(value) => Ok(generate_i32(self.context, value)),
                LiteralValue::String(value) => {
                    let global_value = self.builder.build_global_string_ptr(&value, "")?;
                    Ok(AnyValueEnum::PointerValue(global_value.as_pointer_value()))
                }
            },

            Expr::AssignmentExpr(id, value) => {
                let value = self.generate(*value)?;
                let ty = value.get_type();

                let allocation = match ty {
                    AnyTypeEnum::IntType(ty) => self.builder.build_alloca(ty, &id)?,
                    AnyTypeEnum::PointerType(ty) => self.builder.build_alloca(ty, &id)?,
                    _ => unimplemented!("Unimplemented type"),
                };

                let assign_instruction = match value {
                    AnyValueEnum::IntValue(value) => self.builder.build_store(allocation, value)?,
                    AnyValueEnum::PointerValue(value) => {
                        self.builder.build_store(allocation, value)?
                    }
                    _ => unimplemented!("Unimplemented type for assignment"),
                };

                Ok(assign_instruction.into())
            }

            Expr::FunctionApplication(lhs, rhs) => {
                let function = match *lhs {
                    Expr::Identifier(ref ident) => self
                        .module
                        .get_function(ident)
                        .ok_or_else(|| CompileError::UndefinedFunction(ident.clone()))?,

                    Expr::Literal(_) => Err(CompileError::NotAFunction())?,

                    _ => unimplemented!("FunctionApplication lhs must be an identifier"),
                };

                let arg = self.generate(*rhs)?;

                let result = self.builder.build_call(
                    function,
                    &[arg.try_into().map_err(|_| {
                        CompileError::BasicValueRequired(arg.get_type().to_string())
                    })?],
                    "",
                )?;

                Ok(result.as_any_value_enum())
            }

            Expr::Block(exprs) => exprs
                .into_iter()
                .fold(Err(CompileError::EmptyBlock), |_, expr| self.generate(expr)),

            _ => unimplemented!("Unimplemented expression: {expr:?}"),
        }
    }

    pub fn define_external_functions(&self) {
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
        let i32_type = self.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn_value = self.module.add_function("main", main_fn_type, None);

        let entry_block = self.context.append_basic_block(main_fn_value, "entry");
        self.builder.position_at_end(entry_block);

        let value = self.generate(expr)?;

        match value {
            AnyValueEnum::IntValue(value) => {
                self.builder.build_return(Some(&value))?;
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
