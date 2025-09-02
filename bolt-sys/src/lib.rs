mod bt_object;
mod bt_scalar;
mod context;
pub mod sys;
pub use bt_object::*;
pub use context::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement() {
        let ctx = BoltContext::new();
        let result = ctx.run("let x = 5");
        assert!(result.is_ok());
    }
}
