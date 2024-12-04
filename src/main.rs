mod collect;
mod context;
pub mod driver;
mod io;
mod rule;
mod transform;

fn main() {
    let driver = driver::Driver::new();

    match driver.run() {
        Ok(s) => {
            println!("{}", s);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}
