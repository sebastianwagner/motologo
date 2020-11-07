use std::env::args;
use motologo::parse_params;
use std::fs;
use std::io::{Cursor, BufRead};
use byteorder::{ReadBytesExt, LittleEndian, BigEndian};

fn main() {
    const MOTO_LOGO: &'static[u8; 9] = b"MotoLogo\0";
    const TABLE_ITEM_NAME_LENGTH: u8 = 24;
    const TABLE_MAX_LENGTH: u16 = 0x03ff; // 1024 - 1
    let args: Vec<String> = args().collect();
    let sliced_args: &[String] = &args;
    let (_query, filename) = parse_params(sliced_args);
    //println!("query: {} filename: {:?}.", query, filename);
    let data = fs::read(filename).expect("Unable to read file");
    assert!(data.len() >= 8 + 1 + 2);
    //println!("{:?}", &data[..8]);
    assert_eq!(&data[..=8], MOTO_LOGO);
    let mut rdr = Cursor::new(data);
    rdr.set_position(9);
    let table_padding_start = rdr.read_u16::<LittleEndian>().unwrap();
    println!("table padding start {} 0x{:x?}", table_padding_start, table_padding_start);
    println!("table length {} 0x{:x?}", table_padding_start - 0x0d, table_padding_start - 0x0d);
    assert!(table_padding_start <= TABLE_MAX_LENGTH);
    println!("{} bytes left in table ({} slots)", TABLE_MAX_LENGTH - table_padding_start, (TABLE_MAX_LENGTH - table_padding_start) / 32 );
    rdr.set_position(0xd);
    let table_start :u16 = rdr.position() as u16;
    let mut table_item_start = table_start;
    println!("start: {} item_start: {} padding starts at: {}.", table_start, table_item_start, table_padding_start);
    let mut valid = true;
    let mut last_image_offset: u16 = 0;
    println!("post before loop {} 0x{:x?}", rdr.position(), rdr.position());
    while valid && table_item_start + 32 <= table_padding_start {
        let table_item_start_size = table_item_start as usize;
        let table_item_name_end_size = (table_item_start + TABLE_ITEM_NAME_LENGTH as u16) as usize;
        let mut table_item_name: [u8; TABLE_ITEM_NAME_LENGTH as usize] = Default::default();
        table_item_name.copy_from_slice(&rdr.get_ref()[table_item_start_size..table_item_name_end_size]);
        let table_item_name_str = std::str::from_utf8(&table_item_name).unwrap();
        let table_item_name_str = table_item_name_str.trim_matches(char::from(0));
        println!("{}: {}", table_item_name_str, table_item_start);
        //println!("position in loop {} 0x{:x?} taking {}", rdr.position(), rdr.position(), TABLE_ITEM_NAME_LENGTH);
        rdr.consume(TABLE_ITEM_NAME_LENGTH as usize);
        //println!("position in loop {} 0x{:x?} took {}", rdr.position(), rdr.position(), TABLE_ITEM_NAME_LENGTH);
        let table_item_byte25 = rdr.read_u8().unwrap();
        valid = valid && table_item_byte25 == 0;
        //println!("{:x?} {}", table_item_byte25, table_item_byte25);
        let image_offset = rdr.read_u16::<BigEndian>().unwrap();
        if image_offset <= last_image_offset {
            println!("image offset before last image found until now.");
        }
        last_image_offset = image_offset;
        assert!(image_offset > TABLE_MAX_LENGTH);
        println!("image offset at pos {} 0x{:x?} val 0x{:x?} ({})", rdr.position(), rdr.position(), image_offset, image_offset);
        let table_item_byte28 = rdr.read_u8().unwrap();
        valid = valid && table_item_byte28 == 0;
        let something = rdr.read_u16::<BigEndian>().unwrap();
        //assert!(image_offset > TABLE_MAX_LENGTH);
        println!("something(size?) at pos {} 0x{:x?} val 0x{:x?} ({})", rdr.position(), rdr.position(), something, something);
        //println!("{:x?} {:x?}", &rdr.get_ref()[table_item_name_end_size+4], &rdr.get_ref()[table_item_name_end_size+5]);
        rdr.consume(2);
        //println!("position in loop {} 0x{:x?}", rdr.position(), rdr.position());
        //println!("end of loop position {}", rdr.position() as u16);
        table_item_start += 32;
    }
    assert!(valid);
}
