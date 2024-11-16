mod driver;

fn main() {
    let toml_path = std::env::args().nth(1).expect("invalid toml path");
    let json_path = std::env::args().nth(2).expect("invalid json path");

    let driver = driver::Driver::new(toml_path, json_path);

    match driver.run() {
        Ok(s) => {
            println!("{}", s);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}
