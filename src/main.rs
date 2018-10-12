extern crate redwhite;

use redwhite::ines;

fn main() {
    match ines::Ines::from_file("/Users/jing.peng/Downloads/super-mario-bro.nes") {
        Ok(data) => println!("{:?}", data),
        Err(e) => println!("{}", e),
    };
}
