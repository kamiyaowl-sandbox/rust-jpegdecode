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
        let mut is_soi = false; // start of imageがあったか
        let mut is_eoi = false; // end of imageがあったか

        // DQT 4つの識別子を持ち、8 or 16bitの精度で格納
        let mut quantize_table: [Vec<u16>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), ];

        while !is_eoi {
            match r.read_u16_big_endian() {
                // SOI
                Some(0xffd8) => {
                    is_soi = true;
                    println!("[soi] ffd8");
                },
                // EOI
                Some(0xffd9) => {
                    is_eoi = true;
                    println!("[eoi] ffd9");
                },
                // DQT
                Some(0xffdb) => {
                    let len = r.read_u16_big_endian().unwrap() - 3; // field length+identifier分を引く
                    let identifier = r.read_u8().unwrap();
                    let accuracy_byte = if ((identifier >> 4) & 0x1) == 0x1 { 2 } else { 1 };
                    let table_index = (identifier & 0x03) as usize;
                    println!("[eoi] ffdb len:{} accuracy:{} index:{}", len, accuracy_byte, table_index);
                    // 同じテーブルは定義済のはず
                    assert_eq!(quantize_table[table_index].len(), 0);
                    // 順番に読み出して量子化テーブルに格納
                    let entry_count = (len / accuracy_byte) as usize;
                    for _ in 0..entry_count {
                        let v = match accuracy_byte {
                            1 => r.read_u8().unwrap() as u16,
                            2 => r.read_u16_big_endian().unwrap(),
                            _ => panic!("invalid accuracy in DQT"),
                        };
                        quantize_table[table_index].push(v);
                    }
                    assert_eq!(quantize_table[table_index].len(), entry_count);
                }
                Some(x) if x & 0xff00 == 0xff00 => {
                    let len = r.read_u16_big_endian().unwrap() - 2; // field length分を引く
                    println!("not implemented marker:{:x} len:{}", x, len);
                    // 今は内容を気にしないので読み飛ばす
                    for _ in 0..len {
                        let _ = r.read_u8();
                    }
                },
                Some(x) => {
                    println!("invalid marker {:x}", x);                    
                }
                None => {
                    return Err("unexpected eof".to_owned());
                }
            }
            if !is_soi {
                return Err("bad format".to_owned());
            }
        }

        let dst = Image::new(1 ,1, 1);
        Ok(dst)
    }
}

trait BinaryReader {
    fn read_raw(&mut self, buf: &mut [u8]) -> usize;

    fn read_u8(&mut self) -> Option<u8> {
        let mut read_buf: [u8; 1] = [0; 1];
        match self.read_raw(&mut read_buf) {
            1 => Some(read_buf[0]),
            _ => None
        }
    }
    fn read_u16_big_endian(&mut self) -> Option<u16> {
        let mut read_buf: [u8; 2] = [0; 2];
        match self.read_raw(&mut read_buf) {
            2 => Some(((read_buf[0] as u16) << 8) | (read_buf[1] as u16)),
            _ => None
        }
    }
    fn read_u32_big_endian(&mut self) -> Option<u32> {
        let mut read_buf: [u8; 4] = [0; 4];
        match self.read_raw(&mut read_buf) {
            4 => Some(((read_buf[0] as u32) << 24) | ((read_buf[1] as u32) << 16) | ((read_buf[2] as u32) << 8) | (read_buf[3] as u32)),
            _ => None
        }
    }
}

impl BinaryReader for BufReader<File> {
    fn read_raw(&mut self, buf: &mut [u8]) -> usize {
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
        Ok(img) => {
            println!("parse finish!");
            // TODO: save bmp
        },
        Err(err) => println!("error {}", err)
    }
}
