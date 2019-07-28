use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;

struct JpegImage {
    // SOI
    read_soi        : bool,
    // EOI
    read_eoi        : bool,
    // DQT
    quantize_table  : [Option<[u16; 64]>; 4], // qn
    // SOF
    sof_marker      : Option<u16>,
    height          : Option<usize>,  // y
    width           : Option<usize>,  // x
    components      : Option<usize>,  // Nf
    component_table : [Option<Component>; 4], // Tqn
    // DHT
    huffman_table  : [Option<HuffmanTableInfo>; 4],
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

struct HuffmanTableInfo {
    is_dc            : bool,
    id               : usize,    // 0 ~ 3
    length           : [u8; 16],
    detifnitions     : Vec<u8>, // DC: { databit }, AC: {upper: runlength, lower: databit} 
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
            read_soi: false,
            read_eoi: false,
            quantize_table: [None; 4],
            width: None,
            height: None,
            sof_marker: None,
            components: None,
            component_table: [None; 4],
            huffman_table: [None; 4],
        };

        loop {
            let marker   = r.read_u16_big_endian();
            let is_abort = match marker {
                // SOI
                Some(0xffd8) => {
                    dst.read_soi = true;
                    println!("[SOI] ffd8");
                    false
                },
                // EOI
                Some(0xffd9) => {
                    dst.read_eoi = true;
                    println!("[EOI] ffd9");
                    false
                },
                // DQT
                Some(0xffdb) => {
                    let len = r.read_u16_big_endian().unwrap() - 3; // field length+identifier分を引く
                    let identifier = r.read_u8().unwrap();
                    let accuracy_byte = if ((identifier >> 4) & 0x1) == 0x1 { 2 } else { 1 };
                    let table_index = (identifier & 0x03) as usize;
                    println!("[DQT] ffdb len:{} accuracy:{} index:{}", len, accuracy_byte, table_index);
                    // 順番に読み出して量子化テーブルに格納
                    let mut table = [0; 64];
                    let entry_count = (len / accuracy_byte) as usize;
                    assert_eq!(entry_count, 64);
                    
                    for i in 0..entry_count {
                        let v = match accuracy_byte {
                            1 => r.read_u8().unwrap() as u16,
                            2 => r.read_u16_big_endian().unwrap(),
                            _ => panic!("invalid accuracy in DQT"),
                        };
                        table[i] = v;
                    }
                    assert!(table_index < dst.quantize_table.len());
                    dst.quantize_table[table_index] = Some(table);
                    false
                },
                // DHT
                Some(0xffc4) => {
                    let len = r.read_u16_big_endian().unwrap() - 3; 
                    println!("[DHT] ffc4 len:{}", len);
                    let mut info = HuffmanTableInfo {
                        is_dc        : false,
                        id           : 0,
                        length       : [0; 16],
                        detifnitions : Vec::new(),
                    };
                    let class_info = r.read_u8().unwrap();
                    info.is_dc = if (class_info >> 4) == 0 { true } else { false };
                    info.id    = (class_info & 0xf) as usize;
                    // DCにはテーブルビット配分数は割り当てない
                    let mut length_sum = 0;
                    for i in 0..16 {
                        info.length[i] = r.read_u8().unwrap();
                        length_sum += info.length[i];
                    }
                    for i in 0..length_sum {
                        let data = r.read_u8().unwrap();
                        info.detifnitions.push(data);
                    }
                    println!("[DHT] id:{} is_dc:{}", info.id, info.is_dc);
                    assert!(info.id < 4);
                    dst.huffman_table[info.id] = Some(info);
                    false
                },
                // SOF
                Some(x) if ((0xffc0 <= x) && (x <= 0xffcf) && (x != 0xffc4)) => {
                    dst.sof_marker = Some(x); // ffc0=baseline, ffc2=progressive
                    let len = r.read_u16_big_endian().unwrap() - 8; // ヘッダ部分を除く
                    let precision  = r.read_u8().unwrap() / 8;
                    dst.height     = Some(r.read_u16_big_endian().unwrap() as usize);
                    dst.width      = Some(r.read_u16_big_endian().unwrap() as usize);
                    dst.components = Some(r.read_u8().unwrap() as usize);
                    println!("[SOF] marker:{:x} len:{} width:{} height:{} components:{}", x, len, dst.width.unwrap(), dst.height.unwrap(), dst.components.unwrap());
                    // 成分を順番に読み出す
                    assert_eq!(len as usize, 3 * dst.components.unwrap()); // 1Componentの長さは8byteのはず
                    for i in 0..dst.components.unwrap() {
                        let mut comp = Component {
                            id: ComponentIdentifier::Unknown,
                            w_factor: 0,
                            h_factor: 0,
                            qt_index: 0,
                        };
                        comp.id = match r.read_u8() {
                            Some(1) => ComponentIdentifier::Y,
                            Some(2) => ComponentIdentifier::Cb,
                            Some(3) => ComponentIdentifier::Cr,
                            Some(4) => ComponentIdentifier::I,
                            Some(5) => ComponentIdentifier::Q,
                            _ => panic!("invalid component id"),
                        };
                        let factor: u8 = r.read_u8().unwrap();
                        comp.w_factor = factor >> 4;
                        comp.h_factor = factor & 0xf;
                        comp.qt_index = r.read_u8().unwrap();
                        println!("[SOF] component[{}] = w:{} h:{} qt#:{}", i, comp.w_factor, comp.h_factor, comp.qt_index);
                        dst.component_table[i] = Some(comp);
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
            // 終了セグメントを読んだら終わり(is_abortで止まった場合、read_eoiがない)
            if dst.read_eoi || is_abort {
                break;
            }
        }
        // 後処理
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
            Ok(n) => {
                println!("[BinaryReader] read(len={}) : {:02x?}", buf.len(), buf);
                n
            },
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
