use fxhash::FxHashMap;

use inkwell::AddressSpace;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::BasicValue;
use libc::puts;

use inkwell::OptimizationLevel;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;

use crate::parser::BinaryOp;
use crate::parser::FunType;
use crate::parser::Func;
use crate::parser::Funcall;
use crate::parser::Param;
use crate::parser::Type;
use crate::parser::Var;

pub struct Compiler<'ctx, 'src> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub variables: SymbolHash<'ctx>,
    pub strint: StringInterner<'src>,
}

use crate::parser::Expr;
use crate::parser::Stmt;

use core::panic;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone, PartialEq)]

pub struct Varaibeldata<'ctx> {
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

    // pub fn save(&mut self, name: &'src str) {
    //     if self.string.contains(&name) {
    //         panic!("that name var are saved")
    //     }

    //     self.string.push(name);
    //     self.map.insert(name, self.string.len());
    // }
    pub fn itern(&mut self, name: &'src str) -> usize {
        if let Some(&id) = self.map.get(name) {
            return id;
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
    variebles: FxHashMap<usize, Varaibeldata<'ctx>>,
}
impl<'ctx> SymbolHash<'ctx> {
    pub fn new() -> Self {
        SymbolHash {
            variebles: FxHashMap::default(),
        }
    }

    pub fn save(&mut self, var: Varaibeldata<'ctx>, id: usize) {
        self.variebles.insert(id, var);
    }
    pub fn get_var(&self, id: usize) -> Option<&Varaibeldata<'ctx>> {
        self.variebles.get(&id)
    }
}
impl<'ctx, 'src> Compiler<'ctx, 'src> {
    pub fn new(context: &'ctx Context, module_name: &'src str) -> Self {
        let compiler = Compiler {
            context,
            builder: context.create_builder(),
            module: context.create_module(module_name),
            variables: SymbolHash::new(),
            strint: StringInterner::new(),
        };
        compiler
    }
    fn get_var(&self, name: &str) -> &Varaibeldata<'ctx> {
        let id = self.strint.lookup(name);
        self.variables.get_var(id).unwrap()
    }

    fn get_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Num(_) => Type::Int,

            Expr::Id(name) => self.get_var(name).ty.clone(),

            Expr::Str(_) => Type::Str,

            Expr::Binary(_, _, _) => Type::Int,
            _ => panic!("unknown"),
        }
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
        };
        self.builder.build_store(ptr, value).unwrap();

        let id = self.strint.itern(name);

        self.variables.save(Varaibeldata { ty, ptr }, id);
    }
    // pub fn get_type_our_expr<'a>(&mut self, expr: Expr<'a>) -> Type {
    //     match expr {
    //         Expr::Num(_) => Type::Int,
    //         Expr::Str(_) => Type::Str,
    //         Expr::Id(name) => {
    //             let id = self.strint.lookup(name);
    //             let var = self.variables.get_var(id).unwrap().clone();
    //             var.ty
    //         }
    //         _ => panic!("e"),
    //     }
    // }
    fn parama_to_basic_meta_data(&self, arg: Param) -> BasicMetadataTypeEnum<'ctx> {
        match arg.ty {
            Type::Int => self.context.i64_type().into(),
            Type::Str => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
        }
    }
    // fn parse_params(&mut self, params: Vec<Param>) -> Vec<BasicMetadataTypeEnum> {
    //     let mut paramsa = Vec::new();
    //     for param in params {
    //         let value = self.parama_to_basic_meta_data(param);
    //         paramsa.push(value);
    //     }

    //     paramsa
    // }

    fn main_create(&mut self, func: Func<'src>) {
        let fn_type = self.context.void_type().fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let block = self.context.append_basic_block(function, "start");
        self.builder.position_at_end(block);
        self.strint = StringInterner::new();
        self.variables = SymbolHash::new();

        self.parse_stmts(func.code);
        let _ = self.builder.build_return(None);
    }
    fn func_create(&mut self, func: Func<'src>) {
        let mut params = Vec::new();
        let func_copy = func.clone();
        let fn_type;
        for arg in func.args.iter() {
            let vaule = self.parama_to_basic_meta_data(arg.clone());
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
                FunType::Id(id) => {
                    let id = self.strint.lookup(id);
                    let var = self.variables.get_var(id).unwrap();

                    match var.ty {
                        Type::Int => self.context.i64_type().fn_type(&params, false),
                        Type::Str => {
                            let str_ty = self.context.i8_type().ptr_type(AddressSpace::default());
                            str_ty.fn_type(&params, false)
                        }
                    }
                }
                _ => panic!(""),
            };
        }

        let name = func.name;

        let args = func.args.clone();
        let funcion = self.module.add_function(name, fn_type, None);
        let start_entry = self.context.append_basic_block(funcion, "start");
        self.builder.position_at_end(start_entry);
        self.strint = StringInterner::new();
        self.variables = SymbolHash::new();
        for (i, param) in args.iter().enumerate() {
            let llvm_param = funcion.get_nth_param(i as u32).unwrap();
            llvm_param.set_name(param.name);
            let id = self.strint.itern(param.name);
            let ptr = self
                .builder
                .build_alloca(llvm_param.get_type(), param.name)
                .unwrap();
            self.variables.save(
                Varaibeldata {
                    ty: param.ty.clone(),
                    ptr,
                },
                id,
            );
        }

        self.parse_stmts(func.code.clone());
        if func_copy.ty == None {
            self.builder.build_return(None).unwrap();
        }
    }

    // fn get_pointer_of_expr<'a>(&mut self, expr_raw: *mut Expr<'a>) -> PointerValue<'ctx> {
    //     let expr = unsafe { &*(expr_raw) };
    //     let basic = self.get_value_of_expr(expr.clone()).unwrap();

    //     basic.into_pointer_value()
    // }
    fn get_value_of_expr<'a>(&mut self, expr: Expr<'a>) -> Option<BasicValueEnum<'ctx>> {
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
                return Some(BasicValueEnum::IntValue(result));
            }
            _ => panic!("error expr"),
        }
    }
    fn read_stmt(&mut self, var: Var<'src>) {
        unsafe {
            let expr = &*var.value;
            let expr_type = self.get_expr_type(expr);

            if var.tipe != expr_type {
                panic!("error type")
            }
            let basicvalue = self.get_value_of_expr(expr.clone()).unwrap();

            match basicvalue {
                BasicValueEnum::IntValue(value) => {
                    self.create_var(var.name, BasicValueEnum::IntValue(value), Type::Int);
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
        let mut params = Vec::new();
        for arg in call.args {
            let expr = unsafe { &(*arg) };
            params.push(self.get_value_of_expr(expr.clone()).unwrap().into());
        }
        self.builder.build_call(func, &params, call.name).unwrap();
    }
    fn assign_var(&mut self, name: &'src str, expr: *mut Expr<'src>) {
        unsafe {
            if self.strint.string.contains(&name) {
                let id = self.strint.lookup(name);
                let expr = &mut *expr;
                let expr_type = self.get_expr_type(expr);

                let basic_value = self.get_value_of_expr(expr.clone()).unwrap();
                let var = self.variables.get_var(id).unwrap();
                if var.ty != expr_type {
                    panic!("type error")
                }
                self.builder.build_store(var.ptr, basic_value).unwrap();
            }
        }
    }

    fn binary(&mut self, expr: Expr) -> IntValue<'ctx> {
        match expr {
            Expr::Num(n) => self.context.i64_type().const_int(n as u64, false),

            Expr::Id(id) => {
                let id = self.strint.lookup(id);
                let name = self.strint.string[id];
                let var = self.variables.get_var(id).unwrap();

                let value = self
                    .builder
                    .build_load(self.context.i64_type(), var.ptr, name)
                    .unwrap()
                    .into_int_value();
                return value;
            }
            Expr::Binary(left_raw, op, right_raw) => {
                let left = unsafe { (*left_raw).clone() };
                let right = unsafe { (*right_raw).clone() };

                let left_intv = self.binary(left);
                let right_intv = self.binary(right);

                match op {
                    BinaryOp::Add => self
                        .builder
                        .build_int_add(left_intv, right_intv, "add")
                        .unwrap(),
                    BinaryOp::Sub => self
                        .builder
                        .build_int_sub(left_intv, right_intv, "sub")
                        .unwrap(),
                    BinaryOp::Mul => self
                        .builder
                        .build_int_mul(left_intv, right_intv, "mul")
                        .unwrap(),
                    BinaryOp::Div => self
                        .builder
                        .build_int_unsigned_div(left_intv, right_intv, "div")
                        .unwrap(),
                }
            }

            _ => panic!(),
        }
    }
    fn do_print(&mut self, expr: Expr) {
        let arg = self.get_value_of_expr(expr.clone()).unwrap();
        self.builder
            .build_call(
                self.module.get_function("puts").unwrap(),
                &[arg.into()],
                "puts",
            )
            .unwrap();
    }

    pub fn parse_stmts(&mut self, stmts: Vec<Stmt<'src>>) {
        for stmt in stmts {
            self.parse_stmt(stmt);
        }
    }
    pub fn parse_stmt(&mut self, stmt: Stmt<'src>) {
        match stmt {
            Stmt::Int(var) | Stmt::Str(var) => {
                self.read_stmt(var);
            }
            Stmt::Main(func) => {
                self.main_create(func);
            }
            Stmt::Func(func) => self.func_create(func),
            Stmt::Assign { name, value } => {
                self.assign_var(name, value.clone());
            }
            Stmt::Funcall(call) => {
                if call.name == "puts" {
                    let expr = unsafe { &(*call.args[0]) };

                    self.do_print(expr.clone())
                } else {
                    self.do_call(call)
                }
            }
            Stmt::ReturnStmt(value) => {
                self.do_return(value);
            }
            _ => panic!(""),
        }
    }
}

pub fn run_jit<'ctx>(module: &inkwell::module::Module<'ctx>) {
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let puts_func = module.get_function("puts").unwrap();

    execution_engine.add_global_mapping(&puts_func, puts as usize);

    let main = module.get_function("main").unwrap();
    unsafe {
        execution_engine.run_function(main, &[]);
    }
}
