pub mod cmd;
pub mod conf;
pub mod pkg;
pub mod prelude;

fn main() {
    tracing_subscriber::fmt::init();
}
