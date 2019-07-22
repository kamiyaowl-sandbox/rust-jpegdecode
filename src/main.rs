use std::io;
use std::fs::File;
use std::io::prelude::*;

struct Image {
    width: usize,
    height: usize,
    channels: usize,
    
    // width -> height -> channels
    // self->data[y][x][c]
    data: Vec<Vec<Vec<u8>>>,
}

impl Image {
    fn new(w: usize, h: usize, c: usize) -> Image {
        assert!(w > 0);
        assert!(h > 0);
        assert!(c > 0);

        Image {
            width: w,
            height: h,
            channels: c,
            data: vec![vec![vec![0; c]; w]; h],
        }
    }
    fn clear(&mut self) -> () {
        let pixel = vec![0, 0, 0];
        self.fill(&pixel);
    }
    fn fill(&mut self, pixel: &Vec<u8>) -> () {
        assert!(self.width > 0);
        assert!(self.height > 0);
        assert!(pixel.len() == self.channels);

        for y in 0..self.height {
            for x in 0..self.width {
                for c in 0..self.channels {
                    self.data[y][x][c] = pixel[c];
                }
            }
        }
    }
    fn from_jpeg(binary: &Vec<u8>) -> Result<Image, String> {
        // SOI
        let soiIndex = 0;
        if binary[soiIndex + 0] != 0xff || binary[soiIndex + 1] != 0xd8 {
            return Err("invalid format".to_owned());
        }
        // APP0
        // let app0Index = 2;
        // if binary[app0Index + 0] != 0xff || binary[app0Index + 1] != 0xe0 {
        //     return Err("app0 header error".to_owned());
        // }
        
        let dst = Image::new(0, 0, 3);
        return Ok(dst);
    }
}

trait Reader {
    fn read(&self, buf: &[u8], len: usize) -> usize;
    fn read_u16_big_endian(&self) -> Option<u16> {
        let read_buf: [u8; 2] = [0; 2];
        match self.read(&read_buf, 2) {
            2 => Some(((read_buf[0] as u16) << 8) | (read_buf[1] as u16)),
            _ => None
        }
    }
    fn read_u32_big_endian(&self) -> Option<u32> {
        let read_buf: [u8; 4] = [0; 4];
        match self.read(&read_buf, 4) {
            4 => Some(((read_buf[0] as u32) << 24) | ((read_buf[1] as u32) << 16) | ((read_buf[2] as u32) << 8) | (read_buf[3] as u32)),
            _ => None
        }
    }
}

fn main() -> io::Result<()> {
    // #TODO: 切り替えられるように置き換え
    let src = "/Users/user/Documents/rust-xmodem/test-image/sample1.jpeg";
    // read file
    let mut file = File::open(src)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    println!("Hello, world!");

    let img = Image::from_jpeg(&buf);
    
    Ok(())
}
