use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use pcsc::*;

use super::reader::*;

pub const APOLLO_CARD_TYPE: &'static [u8] = &[
    0x3b, 0xb9, 0x18, 0x0, 0x81, 0x31, 0xfe, 0x9e, 0x80, 0x73, 0xff, 0x61, 0x40, 0x83, 0x0, 0x0,
    0x0, 0xdf,
];


pub struct ApolloCardReader {
}

impl CardReader for ApolloCardReader {
    fn select_aid(&self, _card: &Card, _aid: &[u8]) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }

    fn parse_tlv(&self, buffer: &Vec<u8>) -> Result<HashMap<u16, Vec<u8>>, String> {
        let mut tlvs = HashMap::new();
        let mut offset = 0;
    
        loop {
            let tag = match (&buffer[offset..]).read_u16::<LittleEndian>(){
                Ok(res) => res,
                Err(err) => {
                    return Err(err.to_string());
                }
            };
            let length = match (&buffer[offset + 2..]).read_u16::<LittleEndian>() {
                Ok(res) => res,
                Err(err) => {
                    return Err(err.to_string());
                }
            } as usize;
            offset += 4;
            let end = offset + length;
            tlvs.insert(tag, buffer[offset..end].to_vec());
            offset = end;
    
            if offset >= buffer.len() {
                break;
            }
        }
        Ok(tlvs)
    }
    
    fn select_file(&self, card: &Card, file: &[u8], expected_result_size: u8) -> Result<Vec<u8>, String> {
        let apdu = [0x00, 0xa4, 0x08, 0x00, file.len() as u8, file[0], file[1], expected_result_size];
        let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
        let rapdu = match card.transmit(&apdu, &mut rapdu_buf) {
            Ok(rapdu) => rapdu,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        let result = &rapdu[rapdu.len() - 2..];
        let data = &rapdu[2..rapdu.len() - 2];
    
        if result != SUCCESS_RESPONSE {
            return Err(format!("Reader returned error code {:x?}", result));
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
            return Err(format!("Reader returned error code {:x?}", result));
        }
        Ok(data.to_vec())
    }
    
    fn read_raw_file(&self, card: &Card, file: &[u8], strip_tag: bool) -> Result<Vec<u8>, String> {
        self.select_file(&card, file, 4)?;
    
        let result = self.read_binary(&card, 0, 6)?;
        let mut len: u32 = ((result[5] as u32 & 0xff) << 8) + (result[4] as u32 & 0xff);
        let mut offset = 6;
        if strip_tag {
            len -= 4;
            offset += 4;
        }
    
        let mut buffer: Vec<u8> = vec![];
    
        while len > 0 {
            let mut res = self.read_binary(&card, offset, len as u32)?;
            offset += res.len() as u32;
            len -= res.len() as u32;
            buffer.append(&mut res);
        }
    
        Ok(buffer)
    }
    
}
