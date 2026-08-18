use fxhash::FxHashMap;

use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicValue, FloatValue, FunctionValue};
use inkwell::{AddressSpace, IntPredicate};

use inkwell::OptimizationLevel;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;

use crate::parser::FunType;
use crate::parser::Func;
use crate::parser::Funcall;
use crate::parser::Param;
use crate::parser::Type;
use crate::parser::Var;
use crate::parser::{BinaryOp, IfBlock};
use crate::parser::{CompareOp, WhileBlock};
pub struct Compiler<'ctx, 'src> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub variables: SymbolHash<'ctx>,
    pub string_interner: StringInterner<'src>,
    pub current_func: Option<FunctionValue<'ctx>>,
    pub break_target: Vec<BasicBlock<'ctx>>,
}

use crate::parser::Condition;
use crate::parser::Expr;
use crate::parser::Stmt;
use crate::runtime::{Runtime, runtime_print};
use core::panic;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone, PartialEq)]

pub struct Vardata<'ctx> {
    pub ty: Type,
    ptr: PointerValue<'ctx>,
}

impl<'src> StringInterner<'src> {
    pub fn new() -> Self {
        StringInterner {
            string: Vec::new(),
            map: FxHashMap::default(),
        }
    }

    pub fn itern(&mut self, name: &'src str) -> usize {
        if let Some(&id) = self.map.get(name) {
            id
        } else {
            let id = self.string.len();
            self.string.push(name);
            self.map.insert(name, id);
            id
        }
    }

    pub fn lookup(&self, name: &'src str) -> usize {
        if let Some(id) = self.map.get(name) {
            *id
        } else {
            panic!("cannot find that name")
        }
    }
}
#[derive(Debug, PartialEq, Clone)]

pub struct StringInterner<'src> {
    pub string: Vec<&'src str>,
    pub map: FxHashMap<&'src str, usize>,
}

#[derive(Debug, Clone, PartialEq)]

pub struct SymbolHash<'ctx> {
    vars: FxHashMap<usize, Vardata<'ctx>>,
}
impl<'ctx> SymbolHash<'ctx> {
    pub fn new() -> Self {
        SymbolHash {
            vars: FxHashMap::default(),
        }
    }

    pub fn save(&mut self, var: Vardata<'ctx>, id: usize) {
        self.vars.insert(id, var);
    }
    pub fn get_var(&self, id: usize) -> Option<&Vardata<'ctx>> {
        self.vars.get(&id)
    }
}
impl<'ctx, 'src> Compiler<'ctx, 'src> {
    pub fn new(context: &'ctx Context, module_name: &'src str) -> Self {
        let compiler = Compiler {
            context,
            builder: context.create_builder(),
            module: context.create_module(module_name),
            variables: SymbolHash::new(),
            string_interner: StringInterner::new(),
            current_func: None,
            break_target: vec![],
        };
        compiler
    }
    fn get_var(&self, name: &str) -> &Vardata<'ctx> {
        let id = self.string_interner.lookup(name);
        self.variables.get_var(id).unwrap()
    }

    fn create_var(&mut self, name: &'src str, value: BasicValueEnum<'ctx>, ty: Type) {
        let ptr = match ty {
            Type::Int => self
                .builder
                .build_alloca(self.context.i64_type(), name)
                .unwrap(),
            Type::Str => {
                let ptr_ty = value.into_pointer_value().get_type();
                self.builder.build_alloca(ptr_ty, name).unwrap()
            }
            Type::Float => self
                .builder
                .build_alloca(self.context.f64_type(), name)
                .unwrap(),
        };
        self.builder.build_store(ptr, value).unwrap();

        let id = self.string_interner.itern(name);

        self.variables.save(Vardata { ty, ptr }, id);
    }

    fn param_to_basic_meta_data(&self, arg: Param) -> BasicMetadataTypeEnum<'ctx> {
        match arg.ty {
            Type::Int => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Str => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
        }
    }

    fn main_create(&mut self, func: Func<'src>) {
        let fn_type = self.context.void_type().fn_type(
            &[BasicMetadataTypeEnum::PointerType(
                self.context.i8_type().ptr_type(AddressSpace::default()),
            )],
            false,
        );
        let function = self.module.add_function("main", fn_type, None);
        let block = self.context.append_basic_block(function, "start");
        self.builder.position_at_end(block);
        self.string_interner = StringInterner::new();
        self.variables = SymbolHash::new();
        self.current_func = Some(function);
        self.parse_stmts(func.code);
        let _ = self.builder.build_return(None);
    }
    fn func_create(&mut self, func: Func<'src>) {
        let mut params = Vec::new();
        let func_copy = func.clone();
        let fn_type;
        params.push(BasicMetadataTypeEnum::PointerType(
            self.context.i8_type().ptr_type(AddressSpace::default()),
        ));
        for arg in func.args.iter() {
            let vaule = self.param_to_basic_meta_data(arg.clone());
            params.push(vaule);
        }

        if func.ty == None {
            fn_type = self.context.void_type().fn_type(&params, false)
        } else {
            fn_type = match &func.ty.unwrap() {
                FunType::Int => self.context.i64_type().fn_type(&params, false),
                FunType::Str => {
                    let str_ty = self.context.i8_type().ptr_type(AddressSpace::default());
                    str_ty.fn_type(&params, false)
                }

                FunType::Float => self.context.f64_type().fn_type(&params, false),
            };
        }

        let name = func.name;

        let args = func.args.clone();
        let funcion = self.module.add_function(name, fn_type, None);
        let start_entry = self.context.append_basic_block(funcion, "start");
        self.builder.position_at_end(start_entry);
        self.string_interner = StringInterner::new();
        self.variables = SymbolHash::new();
        let  mut i = 1;
        for  param in args.iter() {
            let llvm_param = funcion.get_nth_param(i as u32).unwrap();
            llvm_param.set_name(param.name);
            let id = self.string_interner.itern(param.name);
            let ptr = self
                .builder
                .build_alloca(llvm_param.get_type(), param.name)
                .unwrap();
            self.builder.build_store(ptr, llvm_param).unwrap();
            self.variables.save(
                Vardata {
                    ty: param.ty.clone(),
                    ptr,
                },
                id,
            );
            i=i+1
        }
        self.current_func = Some(funcion);

        self.parse_stmts(func.code.clone());
        if func_copy.ty == None {
            self.builder.build_return(None).unwrap();
        }
    }
    fn get_runtime(&self) -> PointerValue<'ctx> {
        self.current_func
            .unwrap()
            .get_first_param()
            .unwrap()
            .into_pointer_value()
    }

    fn get_value_of_expr<'a>(&self, expr: Expr<'a>) -> Option<BasicValueEnum<'ctx>> {
        match &expr {
            Expr::Id(name) => {
                let var = self.get_var(name);
                match var.ty {
                    Type::Int => {
                        let value = self
                            .builder
                            .build_load(self.context.i64_type(), var.ptr, "tmp")
                            .unwrap();

                        Some(value)
                    }
                    Type::Float => {
                        let value = self
                            .builder
                            .build_load(self.context.f64_type(), var.ptr, "tmp")
                            .unwrap();
                        Some(value)
                    }
                    Type::Str => {
                        let value = self
                            .builder
                            .build_load(
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                var.ptr,
                                "tmp",
                            )
                            .unwrap();

                        Some(value)
                    }
                }
            }
            Expr::Float(f) => {
                let f_value = self.context.f64_type().const_float(f.clone());
                Some(f_value.as_basic_value_enum())
            }
            Expr::Num(n) => {
                let int_value = self.context.i64_type().const_int(n.clone() as u64, false);
                Some(int_value.as_basic_value_enum())
            }
            Expr::Str(str) => {
                let global = self.builder.build_global_string_ptr(str, "str").unwrap();

                let ptr = unsafe {
                    let i8_type = self.context.i8_type();

                    let array_type = i8_type.array_type(str.len() as u32 + 1);
                    self.builder
                        .build_gep(
                            array_type,
                            global.as_pointer_value(),
                            &[
                                self.context.i64_type().const_zero(),
                                self.context.i64_type().const_zero(),
                            ],
                            "str",
                        )
                        .unwrap()
                };
                Some(ptr.into())
            }
            Expr::Binary(_, _, _) => {
                let result = self.binary(expr);
                Some(result)
            }
            Expr::Func(func) => Some(self.do_call_expr(func.clone())),
            _ => panic!("error expr"),
        }
    }
    fn do_call_expr(&self, call: Funcall) -> BasicValueEnum<'ctx> {
        let func = self.module.get_function(call.name).unwrap();
        let mut params = Vec::new();
        let runtime = self.get_runtime();
        params.push(runtime.into());

        call.args.iter().for_each(|arg| {
            let expr = unsafe { &(**arg) };
            params.push(self.get_value_of_expr(expr.clone()).unwrap().into())
        });
        let res = self.builder.build_call(func, &params, call.name).unwrap();
        res.try_as_basic_value().unwrap_left()
    }
    fn read_stmt(&mut self, var: Var<'src>) {
        unsafe {
            let expr = &*var.value;

            let basicvalue = self.get_value_of_expr(expr.clone()).unwrap();

            match basicvalue {
                BasicValueEnum::IntValue(value) => {
                    self.create_var(var.name, BasicValueEnum::IntValue(value), Type::Int);
                }
                BasicValueEnum::FloatValue(value) => {
                    self.create_var(var.name, BasicValueEnum::FloatValue(value), Type::Float);
                }
                BasicValueEnum::PointerValue(str_poiner) => {
                    self.create_var(
                        var.name,
                        BasicValueEnum::PointerValue(str_poiner),
                        Type::Str,
                    );
                }

                _ => panic!(),
            }
        }
    }
    fn do_return(&mut self, expr: *mut Expr) {
        unsafe {
            let expr = &mut *expr;
            let basic_value = self.get_value_of_expr(expr.clone()).unwrap();
            let _ = self.builder.build_return(Some(&basic_value));
        }
    }
    fn do_call(&mut self, call: Funcall) {
        let func = self.module.get_function(call.name).unwrap();

        let runtime = self.get_runtime();

        let mut params = Vec::new();

        params.push(runtime.into());

        for arg in call.args {
            let expr = unsafe { &(*arg) };

            params.push(self.get_value_of_expr(expr.clone()).unwrap().into());
        }

        self.builder.build_call(func, &params, call.name).unwrap();
    }
    fn assign_var(&mut self, name: &'src str, expr: *mut Expr<'src>) {
        unsafe {
            if self.string_interner.string.contains(&name) {
                let id = self.string_interner.lookup(name);
                let expr = &mut *expr;

                let basic_value = self.get_value_of_expr(expr.clone()).unwrap();
                let var = self.variables.get_var(id).unwrap();

                self.builder.build_store(var.ptr, basic_value).unwrap();
            }
        }
    }

    fn binary(&self, expr: Expr) -> BasicValueEnum<'ctx> {
        match expr {
            Expr::Num(n) => self.context.i64_type().const_int(n as u64, false).into(),
            Expr::Float(n) => self.context.f64_type().const_float(n).into(),

            Expr::Id(id) => {
                let id = self.string_interner.lookup(id);
                let name = self.string_interner.string[id];
                let var = self.variables.get_var(id).unwrap();


                if var.ty == Type::Float {
                    let fvalue = self.builder.build_load(self.context.f64_type(), var.ptr, name).unwrap().into_float_value();
                    fvalue.into()
                } else {
                    let value = self
                        .builder
                        .build_load(self.context.i64_type(), var.ptr, name)
                        .unwrap()
                        .into_int_value();
                    value.into()
                }
            }
            Expr::Binary(left_raw, op, right_raw) => {
                let left = unsafe { (*left_raw).clone() };
                let right = unsafe { (*right_raw).clone() };

                let left_intv = self.binary(left);
                let right_intv = self.binary(right);

                match op {
                    BinaryOp::Add => match (left_intv, right_intv) {
                        (BasicValueEnum::FloatValue(v), BasicValueEnum::IntValue(v2)) => {
                            self
                                .builder
                                .build_float_add(
                                    v,
                                    self.builder
                                        .build_signed_int_to_float(
                                            v2,
                                            self.context.f64_type(),
                                            "int_to_float",
                                        )
                                        .unwrap(),
                                    "add",
                                )
                                .unwrap().into()
                        },

                        (BasicValueEnum::FloatValue(v), BasicValueEnum::FloatValue(v2)) => {
                            self
                                .builder
                                .build_float_add(
                                    v,
                                            v2,
                                    "add"
                                ).unwrap().into()
                        }

                        (BasicValueEnum::IntValue(v), BasicValueEnum::IntValue(v2))=>{
                            self.builder.build_int_add(
                                v,v2 ,"add"
                            ).unwrap().into()
                        }
                        (BasicValueEnum::IntValue(v), BasicValueEnum::FloatValue(v2))=>{
                            self.builder.build_float_add(
                                self.builder.build_signed_int_to_float(v,self.context.f64_type(),"int to float").unwrap() , v2,"add"
                            ).unwrap().into()
                        }
                        _=>panic!()
                    },
                    BinaryOp::Sub => {
                        match (left_intv,right_intv) {
                            (BasicValueEnum::FloatValue(v), BasicValueEnum::IntValue(v2)) => {
                                self
                                    .builder
                                    .build_float_sub(
                                        v,
                                        self.builder
                                            .build_signed_int_to_float(
                                                v2,
                                                self.context.f64_type(),
                                                "int_to_float",
                                            )
                                            .unwrap(),
                                        "sub",
                                    )
                                    .unwrap().into()
                            },

                            (BasicValueEnum::FloatValue(v), BasicValueEnum::FloatValue(v2)) => {
                                self
                                    .builder
                                    .build_float_sub(
                                        v,
                                        v2,
                                        "sub"
                                    ).unwrap().into()
                            }
                            (BasicValueEnum::IntValue(v), BasicValueEnum::FloatValue(v2))=>{
                                self.builder.build_float_sub(
                                    self.builder.build_signed_int_to_float(v,self.context.f64_type(),"int to float").unwrap() , v2,"sub"
                                ).unwrap().into()
                            }
                            (BasicValueEnum::IntValue(v), BasicValueEnum::IntValue(v2)) => {
                                self.builder.build_int_sub(
                                    v,v2 ,"sub"
                                ).unwrap().into()
                            }
                            _=>panic!()
                        }
                    },
                    BinaryOp::Mul =>{
                        match (left_intv,right_intv) {
                            (BasicValueEnum::FloatValue(v), BasicValueEnum::IntValue(v2)) => {
                                self
                                    .builder
                                    .build_float_mul(
                                        v,
                                        self.builder
                                            .build_signed_int_to_float(
                                                v2,
                                                self.context.f64_type(),
                                                "int_to_float",
                                            )
                                            .unwrap(),
                                        "mul",
                                    )
                                    .unwrap().into()
                            },

                            (BasicValueEnum::IntValue(v), BasicValueEnum::FloatValue(v2))=>{
                                self.builder.build_float_mul(
                                    self.builder.build_signed_int_to_float(v,self.context.f64_type(),"int to float").unwrap() , v2,"mul"
                                ).unwrap().into()
                            }
                            (BasicValueEnum::FloatValue(v), BasicValueEnum::FloatValue(v2)) => {
                                self
                                    .builder
                                    .build_float_mul(
                                        v,
                                        v2,
                                        "mul"
                                    ).unwrap().into()
                            }

                            (BasicValueEnum::IntValue(v), BasicValueEnum::IntValue(v2)) => {
                                self.builder.build_int_mul(
                                    v,v2 ,"mul"
                                ).unwrap().into()
                            }
                            _=>panic!()
                        }
                    },
                    BinaryOp::Div => {
                        match (left_intv,right_intv) {
                            (BasicValueEnum::FloatValue(v), BasicValueEnum::IntValue(v2)) => {
                                self
                                    .builder
                                    .build_float_div(
                                        v,
                                        self.builder
                                            .build_signed_int_to_float(
                                                v2,
                                                self.context.f64_type(),
                                                "int_to_float",
                                            )
                                            .unwrap(),
                                        "div",
                                    )
                                    .unwrap().into()
                            },
                            (BasicValueEnum::IntValue(v), BasicValueEnum::FloatValue(v2))=>{
                                self.builder.build_float_div(
                                    self.builder.build_signed_int_to_float(v,self.context.f64_type(),"int to float").unwrap() , v2,"div"
                                ).unwrap().into()
                            }
                            (BasicValueEnum::FloatValue(v), BasicValueEnum::FloatValue(v2)) => {
                                self
                                    .builder
                                    .build_float_div(
                                        v,
                                        v2,
                                        "div"
                                    ).unwrap().into()
                            }

                            (BasicValueEnum::IntValue(v), BasicValueEnum::IntValue(v2)) => {
                                self.builder.build_int_unsigned_div(
                                    v,v2 ,"div"
                                ).unwrap().into()
                            }
                            _=>panic!()
                        }
                    },
                }
            }

            _ => panic!(),
        }
    }

    fn compile_cond(&self, cond: Condition<'src>) -> IntValue<'ctx> {
        println!("{:?}ds", cond.clone());
        match cond {
            Condition::Compare { left, op, right } => self.compile_comp(left, op, right),
            Condition::And(left_raw, right_raw) => {
                let rightv = unsafe { &*right_raw }.clone();
                let leftv = unsafe { &*left_raw }.clone();

                self.do_and(self.compile_cond(leftv), self.compile_cond(rightv))
            }
            Condition::Or(left_raw, right_raw) => {
                let rightv = unsafe { &*right_raw }.clone();
                let leftv = unsafe { &*left_raw }.clone();

                self.do_or(self.compile_cond(leftv), self.compile_cond(rightv))
            }
        }
    }
    fn do_and(&self, left: IntValue<'ctx>, right: IntValue<'ctx>) -> IntValue<'ctx> {
        self.builder.build_and(left, right, "and").unwrap()
    }
    fn do_or(&self, left: IntValue<'ctx>, right: IntValue<'ctx>) -> IntValue<'ctx> {
        self.builder.build_or(left, right, "or").unwrap()
    }
    fn compile_comp(&self, left: *mut Expr, op: CompareOp, right: *mut Expr) -> IntValue<'ctx> {
        let left_expr = unsafe { &*left };
        let right_expr = unsafe { &*right };
        let left_value = self
            .get_value_of_expr(left_expr.clone())
            .unwrap()
            .into_int_value();
        let right_value = self
            .get_value_of_expr(right_expr.clone())
            .unwrap()
            .into_int_value();
        match op {
            CompareOp::Equal => self
                .builder
                .build_int_compare(IntPredicate::EQ, left_value, right_value, "eq")
                .unwrap(),
            CompareOp::NotEqual => self
                .builder
                .build_int_compare(IntPredicate::NE, left_value, right_value, "ne")
                .unwrap(),
            CompareOp::Less => self
                .builder
                .build_int_compare(IntPredicate::SLT, left_value, right_value, "less")
                .unwrap(),

            CompareOp::Greater => self
                .builder
                .build_int_compare(IntPredicate::SGT, left_value, right_value, "greater")
                .unwrap(),
        }
    }

    fn if_create(&mut self, if_block: IfBlock<'src>) {
        let func = self.current_func.unwrap();
        if let Some(else_code) = if_block.elsepart {
            let block_then = self.context.append_basic_block(func, "if_then");
            let block_else = self.context.append_basic_block(func, "if_else");

            let block_end = self.context.append_basic_block(func, "end");
            let cond = self.compile_cond(if_block.cond);

            self.builder
                .build_conditional_branch(cond, block_then, block_else)
                .unwrap();
            self.builder.position_at_end(block_then);
            self.parse_stmts(if_block.code);
            self.builder.build_unconditional_branch(block_end).unwrap();

            self.builder.position_at_end(block_else);
            self.parse_stmts(else_code);
        } else {
            let block_then = self.context.append_basic_block(func, "if_then");
            let block_end = self.context.append_basic_block(func, "if_end");

            let condition = self.compile_cond(if_block.cond);

            self.builder
                .build_conditional_branch(condition, block_then, block_end)
                .unwrap();

            self.builder.position_at_end(block_then);
            self.parse_stmts(if_block.code);

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder.build_unconditional_branch(block_end).unwrap();
            }

            self.builder.position_at_end(block_end);
        }
    }
    pub fn parse_stmts(&mut self, stmts: Vec<Stmt<'src>>) {
        for stmt in stmts {
            self.parse_stmt(stmt);
        }
    }
    fn do_while(&mut self, while_block: WhileBlock<'src>) {
        let func = self.current_func.unwrap();

        let while_cond = self.context.append_basic_block(func, "while_cond");

        let while_body = self.context.append_basic_block(func, "while_body");

        let while_end = self.context.append_basic_block(func, "while_end");

        self.builder.build_unconditional_branch(while_cond).unwrap();

        self.builder.position_at_end(while_cond);

        let cond = self.compile_cond(while_block.cond);

        self.builder
            .build_conditional_branch(cond, while_body, while_end)
            .unwrap();

        self.builder.position_at_end(while_body);

        self.break_target.push(while_end);

        self.parse_stmts(while_block.code);

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(while_cond).unwrap();
        }

        self.break_target.pop();

        self.builder.position_at_end(while_end);
    }
    fn do_break(&mut self) {
        let while_end = self.break_target.last().unwrap();

        self.builder
            .build_unconditional_branch(while_end.clone())
            .unwrap();
    }
    pub fn parse_stmt(&mut self, stmt: Stmt<'src>) {
        match stmt {
            Stmt::Int(var) | Stmt::Str(var) | Stmt::Float(var) => {
                self.read_stmt(var);
            }
            Stmt::Main(func) => {
                self.main_create(func);
            }
            Stmt::If(ifblock) => {
                self.if_create(ifblock);
            }
            Stmt::Func(func) => self.func_create(func),
            Stmt::Assign { name, value } => {
                self.assign_var(name, value.clone());
            }
            Stmt::Break => self.do_break(),
            Stmt::While(while_block) => self.do_while(while_block),
            Stmt::Funcall(call) => self.do_call(call),
            Stmt::ReturnStmt(value) => {
                self.do_return(value);
            }
        }
    }
}
pub fn run_jit<'ctx>(module: &Module<'ctx>, runtime: &mut Runtime) {
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    let print = module.get_function("print").unwrap();

    execution_engine.add_global_mapping(&print, runtime_print as usize);

    type MainFunc = unsafe extern "C" fn(*mut Runtime);

    let addr = execution_engine.get_function_address("main").unwrap();
    let main: MainFunc = unsafe { std::mem::transmute(addr) };

    unsafe {
        main(runtime as *mut Runtime);
    }
}
