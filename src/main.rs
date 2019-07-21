use std::io;
use std::fs::File;
use std::io::prelude::*;

// T=u8: 24bpp rgb
struct RgbImage {
    width: usize,
    height: usize,
    
    rData: Vec<Vec<u8>>,
    gData: Vec<Vec<u8>>,
    bData: Vec<Vec<u8>>,
}

impl RgbImage {
    fn get_pixel(self, x: usize, y: usize) -> (u8, u8, u8) {
        return (self.rData[y][x], self.gData[y][x], self.bData[y][x]);
    }
    fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) -> () {
        self.rData[y][x] = rgb.0;
        self.gData[y][x] = rgb.1;
        self.bData[y][x] = rgb.2;
    }
}

fn decode_jpeg(binary: &Vec<u8>) {
}

fn main() -> io::Result<()> {
    // #TODO: 切り替えられるように置き換え
    let src = "/Users/user/Documents/rust-xmodem/test-image/sample1.jpeg";
    // read file
    let mut file = File::open(src)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    println!("Hello, world!");

    Ok(())
}
