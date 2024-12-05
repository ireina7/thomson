mod collect;
mod context;
pub mod driver;
mod io;
mod rule;
mod transform;

fn main() {
    env_logger::init();
    let driver = driver::Driver::new();

    match driver.run() {
        Ok(s) => {
            println!("{}", s);
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }
}
