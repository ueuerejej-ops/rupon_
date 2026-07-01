use fxhash::FxHashMap;
mod code;

struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    variables: SymbolHash<'ctx>,
    strInt: StringInterner
}


enum valtype{
  Int,
  Value
}
#[derive(Debug, Clone)]
enum InitValue<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
}
use crate::code::ready_code;
use inkwell::OptimizationLevel;
use inkwell::FloatPredicate;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, IntValue};
use std::any::Any;
use std::collections::HashMap;
use std::env::VarError;
use std::num::NonZero;
use std::time::Instant;
use crate::code::Stmt;
use crate::varaibeldata::Int;
#[derive(Debug, Clone)]
enum varaibeldata<'ctx>{
  Float(inkwell::values::PointerValue<'ctx>), 
  Int(inkwell::values::PointerValue<'ctx>)
}
impl StringInterner {
    fn new()->Self{
          StringInterner { strings: Vec::new(), map: FxHashMap::default() }
    }

    fn itern(&mut self,name: &str)->usize{
      if let Some(&id) = self.map.get(name){
        return  id;
      }else{
        let id = self.strings.len();
        self.strings.push(name.to_string());
        self.map.insert(name.to_string(), id);
        id
      }
    }
    fn lookup(&self,id:usize)->&str{
      &self.strings[id]
    }
}
struct StringInterner {
    strings: Vec<String>,
    map: FxHashMap<String, usize>, 
}

struct SymbolHash<'ctx>{
  variebles: FxHashMap<usize,varaibeldata<'ctx>>,
}
impl <'ctx> Compiler<'ctx>{
  fn new(context: &'ctx Context, module_name: &str) -> Self {
     let compiler = Compiler { context, builder: context.create_builder(), module: context.create_module(module_name), variables: SymbolHash::new(),strInt: StringInterner::new() };
         compiler
  }
  fn create_func(&mut self,func:)
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

fn assign_var(&mut self,name: &str,value: InitValue<'ctx>){
  let id = self.strInt.itern(name);
  let var  = self.variables.get_var(id).unwrap();

  match (var,value){
    ( &varaibeldata::Float(ptr),InitValue::Float(val))=>{
      self.builder.build_store(ptr, val);
     }
     ( &varaibeldata::Int(ptr),InitValue::Int(val))=>{
      self.builder.build_store(ptr, val);
     }
    _=>panic!("ewe")
    }
}
fn load_var(&mut self,name: &str)->InitValue<'ctx>{
  
  let id  = self.strInt.itern(name);
  let var  = self.variables.get_var(id).unwrap();
  
  match var{
    &varaibeldata::Int(ptr)=>{
  let value = self.builder
    .build_load(self.context.i64_type(), ptr, "load_tmp")
    .unwrap()
    .into_int_value();
  return  InitValue::Int(value);
    }
    varaibeldata::Float(ptr)=>{
       
       let value = self.builder.build_load(self.context.f64_type(), *ptr, "load_tmp").unwrap().into_float_value();
       return InitValue::Float(value);
    }
  }

}  fn create_var(&mut self,name: &str, value: InitValue<'ctx>){
     match value{
      InitValue::Float(val)=>{
        let f64_type = self.context.f64_type();

        let ptr = self.builder.build_alloca(f64_type, name).unwrap();

        self.builder.build_store(ptr, val).unwrap();

    let id =    self.strInt.itern(name);
    self.variables.save(varaibeldata::Float(ptr), id);
        }
        InitValue::Int(val)=>{
          let int_type = self.context.i64_type();

          let ptr = self.builder.build_alloca(int_type, name).unwrap();
          self.builder.build_store(ptr, val).unwrap();
          let id  = self.strInt.itern(name);
          self.variables.save(varaibeldata::Int(ptr), id);
        }
     }
  }
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


let mut varaebale: Vec<Stmt> =   ready_code(code_my);





  let entry = context.append_basic_block(function, "entry");
 compiler.builder.position_at_end(entry);  

for  var in varaebale{
  match var{
    Stmt::Func(args,code ){
    
    }

    _ => panic!("ERRR")
  }
}



    let ten  = int_type.const_int(10,false);
    compiler.create_var("das", InitValue::Int(ten));



    let res = compiler.load_var("das");

    compiler.assign_var("das", InitValue::Int(int_type.const_int(100, false)));
    compiler.module.print_to_stderr();
    println!("{:?}",res);

}


