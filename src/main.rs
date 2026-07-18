mod code_gen;
mod parser;
mod token;
mod arena;

use inkwell::values::BasicValue;
use inkwell::values::PointerValue;
use inkwell::values::FunctionValue;

use crate::parser::Type;
use crate::parser::Func_call;
use crate::parser::Param;
use crate::code_gen::Compiler;
use crate::parser::Var;
use crate::arena::expr_add;
use crate::token::tokenize;
use crate::token::Token;
use crate::arena::Arena;
use crate::parser::ready_code;
use std::task::Waker;
use crate::parser::Stmt;
use core::panic;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use crate::parser::Expr;
use inkwell::values::{
   BasicValueEnum, 
};


 fn get_type<'a>(compiler: &Compiler,expr: Expr<'a>)->Type{
    match expr{
        Expr::Num(_)=>Type::Int,
        Expr::Str(_)=>Type::Str,
        Expr::Id(name)=>{
         let id = compiler.strint.lookup(name);
         let var = compiler.variables.get_var(id).unwrap().clone();
         var.ty
        } 
        _=>panic!("e")
    }
}



fn main() {
    let mut arena = Arena::new(1000);
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "arm_module");



    let fn_type = context.void_type().fn_type(&[], false);
    let function = compiler.module.add_function("test", fn_type, None);


    let entry = compiler.context.append_basic_block(function, "entry");

    compiler.builder.position_at_end(entry);
    let code_my = r#"  int four = 4 int four_two = four four = four_two

  "#;
let stmt = ready_code(&mut arena as *mut Arena ,code_my,&mut compiler.strint);
compiler.parse_stmt(stmt.clone());
println!("{:?}",stmt);
    compiler.module.print_to_stderr();
}
