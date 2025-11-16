use bolt_rs::*;

//#[derive(BoltObject)]
pub struct TestConfig {
    tooltip: String,
    max: f64,
    min: f64,
}

//#[derive(BoltMod)]
mod bolt_mod {}

//#[derive(BoltFn)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_statement() {
    let mut ctx = Context::new();
    ctx.run("let x = 5").expect("Failed to run statement");
}

#[test]
fn test_into_cstr_trait() {
    let mut ctx = Context::new();

    ctx.run("let x = 1").expect("Failed with &str");
    ctx.run(c"let x = 1").expect("Failed with c-string literal");
}

#[test]
fn test_make_primitive_type() {
    let mut ctx = Context::new();
    ctx.open_all_std();

    let custom_type = ctx
        .make_primitive_type(|_, _| true, "custom")
        .expect("custom type");

    let type_name_value = "custom".make_with_context(&mut ctx);
    ctx.register_type(Value::from_raw(type_name_value), custom_type);

    ctx.run("fn test_custom(x: custom) {}")
        .expect("Failed to define function with custom primitive type");
}

#[test]
fn test_bolt_module() {
    let mut ctx = Context::new();

    let _module = ctx
        .create_module("test_module")
        .expect("Failed to create module");

    let _found = ctx
        .get_module("test_module")
        .expect("Failed to find module");

    ctx.run("import test_module")
        .expect("Failed to import module");
}

#[test]
fn test_bolt_native_fn() {
    let mut ctx = Context::new();
    ctx.open_all_std();

    let module = ctx.make_module();

    extern "C" fn add_numbers(_ctx: *mut sys::bt_Context, thr: *mut sys::bt_Thread) {
        let mut thr_wrap = Thread::from_raw(thr).expect("Null Thread");
        let a = thr_wrap.get_arg::<f64>(0).expect("Bad Arg 0");
        let b = thr_wrap.get_arg::<f64>(1).expect("Bad Arg 1");
        let ret = a + b;
        thr_wrap.return_val(&ret)
    }

    let signature = CallSignature {
        args: vec![f64::make_type(&mut ctx), f64::make_type(&mut ctx)],
        return_ty: f64::make_type(&mut ctx),
    };

    let add_sig = signature.make_type(&mut ctx);

    let native_fn = ctx.make_native(module.clone(), add_sig.clone(), Some(add_numbers));

    // Convert NativeFn to Value by getting its object representation
    let func_value = unsafe {
        let obj_ptr = &(*native_fn.as_ptr()).obj as *const sys::bt_Object as *mut _;
        Value::from_raw(sys::bt_value(obj_ptr))
    };

    let addit_name = "addit".make_with_context(&mut ctx);
    ctx.module_export(
        module.clone(),
        add_sig,
        Value::from_raw(addit_name),
        func_value,
    );

    let module_name = "test_module".make_with_context(&mut ctx);
    ctx.register_module(Value::from_raw(module_name), module);

    ctx.run(
        "import addit from test_module
             addit(1, 5)",
    )
    .expect("Failed to call native function");

    ctx.run(
        "import addit from test_module
             import print, throw from core
             let result = addit(1, 5)
             if result != 6 {
                throw(\"throw\")
             }
            ",
    )
    .expect("Native function returned wrong result");
}
