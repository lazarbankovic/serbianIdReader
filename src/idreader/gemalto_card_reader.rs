use pcsc::*;
use super::reader::*;

pub const GEMALTO_CARD_TYPE: &'static [u8] = &[
    0x3B, 0xFF, 0x94, 0x00, 0x00, 0x81, 0x31, 0x80, 0x43, 0x80, 0x31, 0x80, 0x65, 0xB0, 0x85, 0x02,
    0x01, 0xF3, 0x12, 0x0F, 0xFF, 0x82, 0x90, 0x00, 0x79,
];

pub const GEMALTO_NEW_CARD_TYPE: &'static [u8] = &[
    0x3B, 0xF9, 0x96, 0x00, 0x00, 0x80, 0x31, 0xFE, 0x45, 0x53, 0x43, 0x45, 0x37, 0x20, 0x47, 0x43,
    0x4E, 0x33, 0x5E
];

pub const GEMALTO_EVEN_NEWER_CARD_TYPE: &'static [u8] = &[
    0x3B, 0x9E, 0x96, 0x80, 0x31, 0xFE, 0x45, 0x53, 0x43, 0x45, 0x20, 0x38, 0x2E, 0x30, 0x2D, 0x43, 0x31,
    0x56, 0x30, 0x0D, 0x0A, 0x6F
];

pub const GEMALTO_CARD_TYPE_1: &'static [u8] = &[
    0x3B, 0x9E, 0x96, 0x80, 0x31, 0xFE, 0x45, 0x53, 0x43, 0x45, 0x20, 0x38, 0x2E, 0x30, 0x2D, 0x43, 0x32,
    0x56, 0x30, 0x0D, 0x0A, 0x6C
];

pub const LICNA_KARTA_AID: &[u8] = &[0xF3, 0x81, 0x00, 0x00, 0x02, 0x53, 0x45, 0x52, 0x49, 0x44, 0x01];

pub struct GemaltoCardReader {
}

impl CardReader for GemaltoCardReader  {
    fn select_aid(&self, card: &Card) -> Result<Vec<u8>, String> {
        let apdu: &[u8] = &[0x00, 0xa4, 0x04, 0x00, LICNA_KARTA_AID.len() as u8];
         let apdu = [apdu, LICNA_KARTA_AID].concat();

        let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
        let rapdu = match card.transmit(&apdu, &mut rapdu_buf) {
            Ok(rapdu) => rapdu,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        let result = &rapdu[rapdu.len() - 2..];
        let data = &rapdu[..rapdu.len() - 2];
    
        if result != SUCCESS_RESPONSE {
            return Err(format!("Reader returned error code {:x?}", result));
        }
        Ok(data.to_vec())
    }
    
    fn select_file(&self, card: &Card, file: &[u8], expected_result_size: u8) -> Result<Vec<u8>, String> {
        let apdu = [0x00, 0xa4, 0x08, 0x00, 2 as u8, file[0], file[1], expected_result_size];
        let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
        let rapdu = match card.transmit(&apdu, &mut rapdu_buf) {
            Ok(rapdu) => rapdu,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        let result = &rapdu[rapdu.len() - 2..];
        let data = &rapdu[..rapdu.len() - 2];
    
        if result != SUCCESS_RESPONSE {
            return Err(format!("Reader returned error code at select_file function {:x?}", result));
        }
        Ok(data.to_vec())
    }
    
    fn read_binary(&self, card: &Card, offset: u32, length: u32) -> Result<Vec<u8>, String> {
        let read_size = std::cmp::min(length, BLOCK_SIZE);
        let apdu = [0x00, 0xb0, (offset >> 8) as u8, (offset & 0xff) as u8, read_size as u8];
    
        let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
        let rapdu = match card.transmit(&apdu, &mut rapdu_buf) {
            Ok(rapdu) => rapdu,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        let result = &rapdu[rapdu.len() - 2..];
        let data = &rapdu[..rapdu.len() - 2];
    
        if result != SUCCESS_RESPONSE {
            return Err(format!("Reader returned error code at read_binary function {:x?}", result));
        }
        Ok(data.to_vec())
    }
    
    fn read_raw_file(&self, card: &Card, file: &[u8], strip_tag: bool) -> Result<Vec<u8>, String> {
        let mut buffer: Vec<u8> = vec![];

        self.select_file(&card, file, 4)?;
    
        let len: u32 = 4;
        let mut offset = 0;

        let data = self.read_binary(&card, offset, len)?;
        let mut len = ((data[3] as u32 & 0xff) << 8) + (data[2] as u32 & 0xff);
        offset += data.len() as u32;

        while len > 0 {
            let data = self.read_binary(&card, offset, len)?;

            buffer.append(&mut data.clone());
            offset += data.len() as u32;
            len -= data.len() as u32;
        }
        if strip_tag {buffer.drain(0..4);}
        Ok(buffer)
    }
    
}
