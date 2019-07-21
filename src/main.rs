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
    fn new(w: usize, h: usize) -> RgbImage {
        assert!(w > 0);
        assert!(h > 0);

        RgbImage {
            width: w,
            height: h,
            rData: vec![vec![0; w]; h],
            gData: vec![vec![0; w]; h],
            bData: vec![vec![0; w]; h],
        }
    }
    fn clear(&mut self) -> () {
        self.fill((0, 0, 0));
    }
    fn fill(&mut self, rgb: (u8, u8, u8)) -> () {
        assert!(self.width > 0);
        assert!(self.height > 0);

        for y in 0..self.height {
            for x in 0..self.width {
                self.rData[y][x] = rgb.0;
                self.gData[y][x] = rgb.1;
                self.bData[y][x] = rgb.2;
            }
        }
    }
    fn get_pixel(self, x: usize, y: usize) -> (u8, u8, u8) {
        assert!(x < self.width);
        assert!(y < self.height);
        
        return (self.rData[y][x], self.gData[y][x], self.bData[y][x]);
    }
    fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) -> () {
        assert!(x < self.width);
        assert!(y < self.height);

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

    let img = RgbImage::new(100, 50);
    
    Ok(())
}
