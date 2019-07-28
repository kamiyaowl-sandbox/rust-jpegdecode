use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;

struct JpegImage {
    // DQT
    quantize_table  : [[u16; 64]; 4],
    // SOF
    sof_marker      : Option<u16>,
    height          : Option<usize>,
    width           : Option<usize>,
    components      : Option<usize>,
    component_table : [Component; 4],
}

#[derive(Copy, Clone)]
enum ComponentIdentifier {
    Unknown,
    Y,Cb,Cr,I,Q,
}
#[derive(Copy, Clone)]
struct Component {
    id: ComponentIdentifier,
    h_factor: u8,
    w_factor: u8,
    qt_index: u8,
}

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
        let mut dst = JpegImage {
            quantize_table: [[0; 64]; 4],
            width: None,
            height: None,
            sof_marker: None,
            components: None,
            component_table: [Component{
                id: ComponentIdentifier::Unknown,
                h_factor: 0,
                w_factor: 0,
                qt_index: 0,
            }; 4],
        };
        let mut is_soi = false; // start of imageがあったか
        let mut is_eoi = false; // end of imageがあったか

        while !is_eoi {
            let marker = r.read_u16_big_endian();
            is_eoi = match marker {
                // SOI
                Some(0xffd8) => {
                    is_soi = true;
                    println!("[SOI] ffd8");
                    false
                },
                // EOI
                Some(0xffd9) => {
                    is_eoi = true;
                    println!("[EOI] ffd9");
                    true
                },
                // DQT
                Some(0xffdb) => {
                    let len = r.read_u16_big_endian().unwrap() - 3; // field length+identifier分を引く
                    let identifier = r.read_u8().unwrap();
                    let accuracy_byte = if ((identifier >> 4) & 0x1) == 0x1 { 2 } else { 1 };
                    let table_index = (identifier & 0x03) as usize;
                    println!("[DQT] ffdb len:{} accuracy:{} index:{}", len, accuracy_byte, table_index);
                    // 順番に読み出して量子化テーブルに格納
                    let entry_count = (len / accuracy_byte) as usize;
                    assert_eq!(entry_count, 64);
                    for i in 0..entry_count {
                        let v = match accuracy_byte {
                            1 => r.read_u8().unwrap() as u16,
                            2 => r.read_u16_big_endian().unwrap(),
                            _ => panic!("invalid accuracy in DQT"),
                        };
                        dst.quantize_table[table_index][i] = v;
                    }
                    false
                },
                // DHT
                // Some(0xffc2) => {
                // },
                // SOF
                Some(x) if ((0xffc0 <= x) && (x <= 0xffcf) && (x != 0xffc4)) => {
                    dst.sof_marker = Some(x); // ffc0=baseline, ffc2=progressive
                    let len = r.read_u16_big_endian().unwrap() - 8; // field length分を引く
                    let precision  = r.read_u8().unwrap() / 8;
                    dst.height     = Some(r.read_u16_big_endian().unwrap() as usize);
                    dst.width      = Some(r.read_u16_big_endian().unwrap() as usize);
                    dst.components = Some(r.read_u8().unwrap() as usize);
                    println!("[SOF] marker:{:x} len:{} width:{} height:{} components:{}", x, len, dst.width.unwrap(), dst.height.unwrap(), dst.components.unwrap());
                    // 成分を順番に読み出す
                    assert_eq!(len as usize, 3 * dst.components.unwrap()); // 1Componentの長さは8byteのはず
                    for i in 0..dst.components.unwrap() {
                        dst.component_table[i].id = match r.read_u8() {
                            Some(1) => ComponentIdentifier::Y,
                            Some(2) => ComponentIdentifier::Cb,
                            Some(3) => ComponentIdentifier::Cr,
                            Some(4) => ComponentIdentifier::I,
                            Some(5) => ComponentIdentifier::Q,
                            _ => panic!("invalid component id"),
                        };
                        let factor: u8 = r.read_u8().unwrap();
                        dst.component_table[i].w_factor = factor >> 4;
                        dst.component_table[i].h_factor = factor & 0xf;
                        dst.component_table[i].qt_index = r.read_u8().unwrap();
                        println!("component[{}] = w:{} h:{} qt#:{}", i, dst.component_table[i].w_factor, dst.component_table[i].h_factor, dst.component_table[i].qt_index);
                    }
                    false
                }
                Some(x) => {
                    let len = r.read_u16_big_endian().unwrap() - 2; // field length分を引く
                    println!("not implemented marker:{:x} len:{}", x, len);
                    // 今は内容を気にしないので読み飛ばす
                    for _ in 0..len {
                        let _ = r.read_u8();
                    }
                    false
                },
                None => {
                    panic!("unexpected eof");
                },
            };
            if !is_soi {
                panic!("bad format(marker 'SOI' not found)");
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
