fn main() {
    let driver = thomson::driver::Driver::new();

    match driver.run() {
        Ok(s) => {
            println!("{}", s);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}
