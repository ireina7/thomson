mod collect;
mod component;
mod io;
mod transform;

use component::driver;

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
