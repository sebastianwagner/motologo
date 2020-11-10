use std::env::args;
use motologo::parse_params;
use std::fs;
use std::io::{Cursor, Read};
use byteorder::{ReadBytesExt, LittleEndian};
struct TableDescriptor {
    #[allow(dead_code)]
    name: String,
    offset: u32,
    #[allow(dead_code)]
    length: u32,
}
/**
 * @thx @ https://github.com/eriktim/moto-bootlogo.git; mlichvar et. al
 */
fn main() {
    const MOTO_LOGO: &'static[u8; 9] = b"MotoLogo\0";
    const TABLE_ITEM_NAME_LENGTH: usize = 24;
    const TABLE_ITEM_LENGTH :u8 = 32;
    const TABLE_START: u8 = 0x0d;
    const TABLE_MAX_LENGTH: u16 = 0x03ff; // 1024 - 1
    fn read_table_descriptor_name(reader: &mut impl Read) -> String {
        let buffer = &mut [0; TABLE_ITEM_NAME_LENGTH];
        let _read = reader.read(buffer);
        let buffer_str = std::str::from_utf8(buffer).unwrap();
        let buffer_str_trim = buffer_str.trim_matches(b'\0' as char);
        let mut buf = String::with_capacity(buffer_str_trim.len());
        buf.push_str(&buffer_str_trim);
        buf
    }
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
    let table_padding_start = rdr.read_u32::<LittleEndian>().unwrap();
    let table_length = table_padding_start - TABLE_START as u32;
    let table_padding_length = TABLE_MAX_LENGTH as u32 - table_padding_start;
    let table_item_count = (table_length / TABLE_ITEM_LENGTH as u32) as u16;
    let mut table_items :Vec<TableDescriptor> = Vec::with_capacity(table_item_count as usize);
    let table_padding_slot_count = (table_padding_length / TABLE_ITEM_LENGTH as u32) as u16;
    println!("# table 0xff padding start {} 0x\"{:04x?}\"(le)", table_padding_start, table_padding_start.to_be());
    print!("# table length {} bytes with {} slots", table_length, table_item_count);
    println!(" {} bytes left (~{} slots)", table_padding_length, table_padding_slot_count);
    assert!(table_padding_start <= TABLE_MAX_LENGTH.into());
    let mut table_item_start :u32 = TABLE_START.into();
    println!();
    let mut last_image_offset = 0;
    let mut last_image_end = table_padding_start.into();
    while (table_item_start + TABLE_ITEM_LENGTH as u32) <= table_padding_start {
        let table_item_start_size = table_item_start as usize;
        let table_item_name_end_size = (table_item_start + TABLE_ITEM_NAME_LENGTH as u32) as usize;
        let mut table_item_name: [u8; TABLE_ITEM_NAME_LENGTH] = Default::default();
        table_item_name.copy_from_slice(&rdr.get_ref()[table_item_start_size..table_item_name_end_size]);
        let table_item_name_str = std::str::from_utf8(&table_item_name).unwrap();
        let _table_item_name_str = table_item_name_str.trim_matches(char::from(0));
        let read_buffer = read_table_descriptor_name(&mut rdr);
        let image_offset = rdr.read_u32::<LittleEndian>().unwrap();
        assert!(image_offset > TABLE_MAX_LENGTH.into());
        // address byte aligned?
        assert_eq!(0, image_offset % 256);
        if last_image_end > table_padding_start.into() {
            // info for last image
            println!(" with {:3} bytes left", image_offset - last_image_end);
        }
        print!("{:24}", read_buffer);
        if image_offset <= last_image_offset {
            println!("### image offset before last image offset ###");
        } else if image_offset <= last_image_end {
            println!("### image offset before last image ended ###");
        }
        last_image_offset = image_offset;
        let image_length = rdr.read_u32::<LittleEndian>().unwrap();
        let table_item = TableDescriptor {
            name: read_buffer,
            offset: image_offset,
            length: image_length,
        };
        print!(" # offset byte {:7} 0x\"{:08x}\"(le in hexdump) at position 0x{:08x}(be)", table_item.offset, table_item.offset.to_be(), table_item.offset);
        table_items.push(table_item);
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
