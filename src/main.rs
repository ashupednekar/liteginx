pub mod cmd;
pub mod pkg;
pub mod conf;
pub mod prelude;

fn main() {
    tracing_subscriber::fmt::init();
}
