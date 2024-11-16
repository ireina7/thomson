fn main() {
    let toml_path = std::env::args().nth(1).expect("invalid toml path");
    let json_path = std::env::args().nth(2).expect("invalid json path");

    let driver = thomson::Driver {
        json_path,
        toml_path,
    };

    match driver.run() {
        Ok(s) => {
            println!("{}", s);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}
