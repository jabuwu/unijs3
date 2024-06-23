use tracing::{info, Level};
use unijs3::{Function, Value};

fn main() {
    unilog::init(Level::INFO, "");
    let add = Function::new(|args| {
        let a = args.get(0).into_number().unwrap();
        let b = args.get(1).into_number().unwrap();
        Value::Number(a + b)
    });
    let result = add.call([3.0.into(), 4.0.into()]);
    info!("{}", result);
}
