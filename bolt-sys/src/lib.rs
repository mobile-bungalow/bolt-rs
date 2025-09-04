mod bt_value;
mod context;
mod error;
pub mod sys;
pub use bt_value::{bt_object, bt_scalar};
pub use context::*;
pub use error::*;

#[cfg(test)]
mod tests {
    use bt_value::{CallSignature, ScalarTypeSignature};
    use sys::{bt_Context, bt_Thread};

    use super::*;

    #[test]
    fn test_statement() {
        let ctx = BoltContext::new();
        let result = ctx.run("let x = 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bolt_module() {
        let mut ctx = BoltContext::new();

        let module = ctx.make_module().expect("Failed to create module");

        ctx.register_module("test_module", &module)
            .expect("Failed to register module");

        let found_module = ctx
            .find_module("test_module")
            .expect("Failed to find module");

        assert_eq!(found_module.name(), Some("test_module"));

        let result = ctx.run("import test_module");

        assert!(result.is_ok());
    }

    #[test]
    fn test_bolt_native_fn() {
        let mut ctx = BoltContext::new();
        ctx.open_all_std();

        let module = ctx.make_module().expect("Failed to create module");

        extern "C" fn add_numbers(_ctx: *mut bt_Context, thr: *mut bt_Thread) {
            let mut thr_wrap = BoltThread::from_raw(thr).expect("Null Thread");
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

        let func = ctx.make_native_function(&module, &add_sig, add_numbers);

        ctx.module_export(&module, add_sig, "addit", func);

        ctx.register_module("test_module", &module)
            .expect("Failed to register module");

        let result = ctx.run(
            "import addit from test_module
             addit(1, 5)",
        );

        assert!(result.is_ok());

        let wrong_result = ctx.run(
            "import addit from test_module
             import print, throw from core
             let result = addit(1, 5)
             if result != 6 {
                throw(\"throw\")
             }
            ",
        );
        assert!(wrong_result.is_ok());
    }
}
