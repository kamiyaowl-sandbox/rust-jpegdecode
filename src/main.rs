use std::io;
use std::io::BufReader;
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
    fn from_jpeg(r: &mut BinaryReader) -> Result<Image, String> {
        let mut dst = Image::new(1 ,1, 1);
        let mut is_soi = false;
        let mut is_eoi = false;
        while !is_eoi {
            match r.read_u16_big_endian() {
                // SOI
                Some(0xffd8) => {
                    is_soi = true;
                },
                // EOI
                Some(0xffd9) => {
                    is_eoi = true;
                },
                Some(x) => {
                    panic!("not implemented marker {:x}", x);
                },
                None => {
                    return Err("unexpected eof".to_owned());
                }
            }
            if !is_soi {
                return Err("bad format".to_owned());
            }
        }
        Ok(dst)
    }
}

trait BinaryReader {
    fn read_u8(&mut self, buf: &mut [u8]) -> usize;
    fn read_u16_big_endian(&mut self) -> Option<u16> {
        let mut read_buf: [u8; 2] = [0; 2];
        match self.read_u8(&mut read_buf) {
            2 => Some(((read_buf[0] as u16) << 8) | (read_buf[1] as u16)),
            _ => None
        }
    }
    fn read_u32_big_endian(&mut self) -> Option<u32> {
        let mut read_buf: [u8; 4] = [0; 4];
        match self.read_u8(&mut read_buf) {
            4 => Some(((read_buf[0] as u32) << 24) | ((read_buf[1] as u32) << 16) | ((read_buf[2] as u32) << 8) | (read_buf[3] as u32)),
            _ => None
        }
    }
}

impl BinaryReader for BufReader<File> {
    fn read_u8(&mut self, buf: &mut [u8]) -> usize {
        match self.read(buf) {
            Ok(n) => n,
            Err(_) => 0,
        }
    }    
}

fn main() {
    // #TODO: 切り替えられるように置き換え
    let src = "/Users/user/Documents/rust-xmodem/test-image/sample1.jpeg";
    let mut reader = BufReader::new(File::open(src).unwrap());

    match Image::from_jpeg(&mut reader) {
        Ok(img) => println!("parse finish!"),
        Err(err) => panic!(err)
    }
}
