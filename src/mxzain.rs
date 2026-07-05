use fxhash::FxHashMap;
mod parser;
mod token;
use inkwell::types;
use inkwell::values::InstructionValue;
use inkwell::values::PointerValue;
use parser::Type;
use inkwell::values::BasicValue;
struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    variables: SymbolHash<'ctx>,
    strint: StringInterner<'ctx>
}



#[derive(Debug, Clone)]
enum InitValue<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
}
use crate::parser::Expr;
use crate::parser::Stmt;
use crate::parser::ready_code;
use inkwell::OptimizationLevel;
use inkwell::FloatPredicate;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, IntValue};
use core::panic;
use std::any::Any;
use std::collections::HashMap;
use std::env::VarError;
use std::time::Instant;
use crate::parser::Var;
#[derive(Debug, Clone)]
struct varaibeldata<'ctx>{
  ty: Type,
  ptr: PointerValue<'ctx>
}

impl<'a> StringInterner<'a> {
    fn new()->Self{
          StringInterner { string: Vec::new(), map: FxHashMap::default() }
    }

    fn itern(&mut self,name: &'a str)->usize{
      if let Some(&id) = self.map.get(name){
        return  id;
      }else{
        let id = self.string.len();
        self.string.push(name);
        self.map.insert(name, id);
        id
      }
    }
    fn lookup(&self,name: &'a str)->&usize{
     if let Some(id) =  self.map.get(name){
        id
     }else{
      panic!("cannot find that name")
     }
    }
    
   
}
struct StringInterner<'a> {
    string: Vec<&'a str>,
    map: FxHashMap<&'a str, usize>, 
}

struct SymbolHash<'ctx>{
  variebles: FxHashMap<usize,varaibeldata<'ctx>>,
}
impl <'ctx>SymbolHash<'ctx>{
  fn new()->Self{
    SymbolHash { variebles: FxHashMap::default()}
  }

  fn save(&mut self,var: varaibeldata<'ctx>,id: usize){
    self.variebles.insert(id,var);
  }
  fn get_var(&self,id: usize)->Option<&varaibeldata<'ctx>>{
self.variebles.get(&id)
  }
}
impl <'ctx> Compiler<'ctx>{
  fn new(context: &'ctx Context, module_name: &str) -> Self {
     let compiler = Compiler { context, builder: context.create_builder(), module: context.create_module(module_name), variables: SymbolHash::new(),strint: StringInterner::new() };
         compiler
  }
fn func_call(&self,func: FunctionValue<'ctx>,name: &str,args: Option<Vec<InitValue<'ctx>>>)->Option<Vec<InitValue<'ctx>>>{
     let mut llvm_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();

     if let Some(args) = args{
      for arg in  args{
        match arg{
        InitValue::Float(val)=>llvm_args.push(val.into()),
        InitValue::Int(val) => llvm_args.push(val.into()),
        }
      }
     }

     let call = self.builder.build_call(func, &llvm_args, name).unwrap();
     if let Some(basic_val) = call.try_as_basic_value().left() {
      let mut vec_of_return: Vec<InitValue<'ctx>> = Vec::new();
            if basic_val.is_float_value() {
              vec_of_return.push(InitValue::Float(basic_val.into_float_value()));
                Some(vec_of_return)
            } else if basic_val.is_int_value() {
               vec_of_return.push(InitValue::Int(basic_val.into_int_value()));
               Some(vec_of_return)
            } else {
                None 
            }
        } else {
            None
        }
}

fn assign_var(&mut self,name: &'ctx str,val: BasicValueEnum){
  let id = self.strint.itern(name);
  let var  = self.variables.get_var(id).unwrap();

   self.builder.build_store(var.ptr, val).unwrap();

}
fn load_var(&mut self,name: &'ctx str)->InitValue{
  
  let id  = self.strint.itern(name);
  let var  = self.variables.get_var(id).unwrap();
  
  match var.ty{
    Type::Int=>{
  let value = self.builder
    .build_load(self.context.i64_type(), var.ptr, "load_tmp")
    .unwrap()
    .into_int_value();
  
  InitValue::Int(value)
    }
   _=>panic!("")
  }

}

fn create_var<'a>(&mut self,name: &'ctx str, stmt: Stmt<'a>){
     match stmt{
   Stmt::Int(value)=>{
    let i64_type = self.context.i64_type();

    let ptr = self.builder.build_alloca(i64_type, name).unwrap();
    let value = self.get_value_of_expr(value.expr).unwrap();
    self.builder.build_store(ptr, value).unwrap();


    let id = self.strint.itern(name);

    self.variables.save(varaibeldata { ty:Type::Int, ptr }, id);
   }
   _=>panic!("jwdwsd")
     }
  }

  fn get_value_of_expr<'a>(&mut self,expr: Expr<'a>)->Option<BasicValueEnum<'ctx>>{
        match expr{
          Expr::Num(n)=>{
               let int_value = self.context.i64_type().const_int(n as u64, false);
              Some( int_value.as_basic_value_enum())
          },
           _=>panic!("error expr")
        }
  }
}

fn main(){
  let context = Context::create();
let mut compiler = Compiler::new(&context, "arm_module");

  let f64_type = context.f64_type();
  let int_type = context.i64_type();
  
  let fn_type = context.void_type().fn_type(&[], false);
  let function = compiler.module.add_function("test", fn_type, None);

  let code_my = "func hello(){
  int i = 0
}
  ";


let mut varaebale =   ready_code(code_my);




  let entry = context.append_basic_block(function, "entry");
 compiler.builder.position_at_end(entry);  



    let ten  = int_type.const_int(10,false);
    compiler.create_var("das", Stmt::Int(Var { tipe: Type::Int, name: "hsk", expr: Expr::Num(10) }));
    let ten3  = int_type.const_int(13,false);


    let res = compiler.load_var("das");

    compiler.assign_var("das", BasicValueEnum::IntValue(ten3));
    compiler.module.print_to_stderr();

}


fn add_var_to_hashmap<'a>(name: &'a str,ty: Type,value: Expr){

}


