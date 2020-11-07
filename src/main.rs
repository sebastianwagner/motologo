use std::env::args;
use motologo::parse_params;
use std::fs;
use std::io::{Cursor, BufRead};
use byteorder::{ReadBytesExt, LittleEndian};

/**
 * @thx @ https://github.com/eriktim/moto-bootlogo.git; mlichvar et. al
 */
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
    println!("# table padding start {} 0x\"{:04x?}\"(le)", table_padding_start, table_padding_start.to_be());
    print!("# table length {} with {} slots", table_padding_start - 0x0d, (table_padding_start - 0x0d)/32);
    assert!(table_padding_start <= TABLE_MAX_LENGTH);
    println!(" {} bytes left ({} slots)", TABLE_MAX_LENGTH - table_padding_start, (TABLE_MAX_LENGTH - table_padding_start) / 32 );
    rdr.set_position(0xd);
    let table_start :u16 = rdr.position() as u16;
    let mut table_item_start = table_start;
    println!();
    let mut last_image_offset = 0;
    let mut last_image_end = table_padding_start.into();
    while table_item_start + 32 <= table_padding_start {
        let table_item_start_size = table_item_start as usize;
        let table_item_name_end_size = (table_item_start + TABLE_ITEM_NAME_LENGTH as u16) as usize;
        let mut table_item_name: [u8; TABLE_ITEM_NAME_LENGTH as usize] = Default::default();
        table_item_name.copy_from_slice(&rdr.get_ref()[table_item_start_size..table_item_name_end_size]);
        let table_item_name_str = std::str::from_utf8(&table_item_name).unwrap();
        let table_item_name_str = table_item_name_str.trim_matches(char::from(0));
        rdr.consume(TABLE_ITEM_NAME_LENGTH as usize);
        let image_offset = rdr.read_u32::<LittleEndian>().unwrap();
        assert!(image_offset > TABLE_MAX_LENGTH.into());
        // address byte aligned?
        assert_eq!(0, image_offset % 256);
        if last_image_end > table_padding_start.into() {
            // info for last image
            println!(" with {:3} bytes left", image_offset - last_image_end);
        }
        print!("{:24}", table_item_name_str);
        if image_offset <= last_image_offset {
            println!("### image offset before last image offset ###");
        } else if image_offset <= last_image_end {
            println!("### image offset before last image ended ###");
        }
        last_image_offset = image_offset;
        print!(" # offset byte {:7} 0x\"{:08x}\"(le in hexdump) at position 0x{:08x}(be)", image_offset, image_offset.to_be(), image_offset);
        let image_length = rdr.read_u32::<LittleEndian>().unwrap();
        print!(", length {:6} bytes seen as hexdump \"0x{:08x?}\"(le)", image_length, image_length.to_be());
        let image_end = image_offset + image_length;
        last_image_end = image_end;
        print!(", image end at byte 0x{:08x?}", image_end);
        // all remaining bytes in table padding until first image are 0xff
        table_item_start += 32;
    }
    println!();
    // all remaining bytes for padding after last image are zero
}
