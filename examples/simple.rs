use tracing::{info, Level};
use unijs3::{eval, global_set, Function, OrThrow};

fn main() {
    unilog::init(Level::INFO, "");
    let add = Function::new(|args| {
        let x = args
            .get(0)
            .into_number()
            .or_throw("Expected first argument to be a number.")?;
        let y = args
            .get(1)
            .into_number()
            .or_throw("Expected second argument to be a number.")?;
        Ok(x + y)
    });
    global_set("add", add);
    let result = eval("add(3, 5)");
    info!("{:?}", result);
}
