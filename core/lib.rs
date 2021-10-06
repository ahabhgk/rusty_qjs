pub mod extension;
pub mod qjs;

#[cfg(test)]
mod tests {
    use crate::qjs;

    #[test]
    fn eval() {
        let ctx = qjs::Context::new().unwrap();
        ctx.eval(
            "function main() {\n    const a = 1\n    const b = 2\n    return a + b\n}\n\nmain()\n",
        )
        .unwrap();
    }
}
