use byteorder::{LittleEndian, ReadBytesExt};
use pcsc::*;
use std::{
    collections::HashMap,
    fmt::{self},
};

const BLOCK_SIZE: u32 = 254;
const PERSONAL_FILE: &'static [u8] = &[0x0F, 0x03];
const DOCUMENT_FILE: &'static [u8] = &[0x0F, 0x02];
const RESIDENCE_FILE: &'static [u8] = &[0x0F, 0x04];
const PHOTO_FILE: &'static [u8] = &[0x0F, 0x06];
const APOLLO_CARD_TYPE: &'static [u8] = &[
    0x3b, 0xb9, 0x18, 0x0, 0x81, 0x31, 0xfe, 0x9e, 0x80, 0x73, 0xff, 0x61, 0x40, 0x83, 0x0, 0x0,
    0x0, 0xdf,
];
const GEMALTO_CARD_TYPE: &'static [u8] = &[
    0x3B, 0xFF, 0x94, 0x00, 0x00, 0x81, 0x31, 0x80, 0x43, 0x80, 0x31, 0x80, 0x65, 0xB0, 0x85, 0x02,
    0x01, 0xF3, 0x12, 0x0F, 0xFF, 0x82, 0x90, 0x00, 0x79,
];
const SUCCESS_RESPONSE: &'static [u8]= &[0x90, 0x00];
const LICNA_KARTA_AID: &[u8] = &[0xF3, 0x81, 0x00, 0x00, 0x02, 0x53, 0x45, 0x52, 0x49, 0x44, 0x01];

#[derive(Eq, Hash, PartialEq)]
#[derive(Clone, Copy)]
pub enum PersonalIdTag {
    DocRegNo = 1546,
    IssuingDate = 1549,
    ExpiryDate = 1550,
    IssuingAuthority = 1551,
    PersonalNumber = 1558,
    Surname = 1559,
    GivenName = 1560,
    ParentGivenName = 1561,
    Sex = 1562,
    PlaceOfBirth = 1563,
    CommunityOfBirth = 1564,
    StateOfBirth = 1565,
    DateOfBirth = 1566,
    State = 1568,
    Community = 1569,
    Place = 1570,
    Street = 1571,
    HouseNumber = 1572,
    HouseLetter = 1573,
    Entrance = 1574,
    Floor = 1575,
    AppartmentNumber = 1578,
    AddressDate = 1580,
    None = 0,
}

pub struct PersonalIdTagType {
    pub id: PersonalIdTag,
    pub description: &'static str,
    pub json_id: &'static str
}

const ID_TAGS: &[PersonalIdTagType] = 
        &[PersonalIdTagType{id: PersonalIdTag::DocRegNo, description: "Broj dokumenta", json_id: "DocRegNo"},
          PersonalIdTagType{id: PersonalIdTag::IssuingDate, description: "Datum izdavanja", json_id: "IssuingDate"},
          PersonalIdTagType{id: PersonalIdTag::ExpiryDate, description: "Datum isteka", json_id: "ExpiryDate"},
          PersonalIdTagType{id: PersonalIdTag::IssuingAuthority, description: "Izdato od", json_id: "IssuingAuthority"},
          PersonalIdTagType{id: PersonalIdTag::PersonalNumber, description: "JMBG", json_id: "PersonalNumber"},
          PersonalIdTagType{id: PersonalIdTag::Surname, description: "Prezime", json_id: "Surname"},
          PersonalIdTagType{id: PersonalIdTag::GivenName, description: "Ime", json_id: "GivenName"},
          PersonalIdTagType{id: PersonalIdTag::ParentGivenName, description: "Ime roditelja", json_id: "ParentGivenName"},
          PersonalIdTagType{id: PersonalIdTag::Sex, description: "Pol", json_id: "Sex"},
          PersonalIdTagType{id: PersonalIdTag::PlaceOfBirth, description: "Mesto rodjenja", json_id: "PlaceOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::CommunityOfBirth, description: "Opstina rodjenja", json_id: "CommunityOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::StateOfBirth, description: "Drzava rodjenja", json_id: "StateOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::DateOfBirth, description: "Datum rodjenja", json_id: "DateOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::State, description: "Drzava", json_id: "State"},
          PersonalIdTagType{id: PersonalIdTag::Community, description: "Opstina", json_id: "Community"},
          PersonalIdTagType{id: PersonalIdTag::Place, description: "Mesto", json_id: "Place"},
          PersonalIdTagType{id: PersonalIdTag::Street, description: "Ulica", json_id: "Street"},
          PersonalIdTagType{id: PersonalIdTag::HouseNumber, description: "Kucni broj", json_id: "HouseNumber"},
          PersonalIdTagType{id: PersonalIdTag::HouseLetter, description: "Kucna oznaka", json_id: "HouseLetter"},
          PersonalIdTagType{id: PersonalIdTag::Entrance, description: "Broj ulaza", json_id: "Entrance"},
          PersonalIdTagType{id: PersonalIdTag::Floor, description: "Sprat broj", json_id: "Floor"},
          PersonalIdTagType{id: PersonalIdTag::AppartmentNumber, description: "Broj stana", json_id: "AppartmentNumber"},
          PersonalIdTagType{id: PersonalIdTag::AddressDate, description: "Datum adrese", json_id: "AddressDate"},
          PersonalIdTagType{id: PersonalIdTag::None, description: "", json_id: ""},
         ];

pub struct PersonalIdItem {
    pub tag: &'static PersonalIdTagType,
    pub value: String,
}

impl PersonalIdItem {
    fn new(tag: &'static PersonalIdTagType, map: &HashMap<u16, Vec<u8>>) -> Option<PersonalIdItem> {
        let val = map.get(&(tag.id as u16))?;
        let s = match std::str::from_utf8(&val) {
            Ok(v) => v,
            Err(_) => {return Option::None}//panic!("Invalid UTF-8 sequence: {}", e),
        };
        Some(PersonalIdItem {tag: tag, value: s.to_string()})
    }
}

pub struct PersonalId {
    pub personal: HashMap<PersonalIdTag, PersonalIdItem>,
    pub image: Vec<u8>,
    card_reader: Box<dyn CardReader>,
}

impl PersonalId {
    pub fn new(card: &Card) -> core::result::Result<PersonalId, String> {
        let buffer = match card.get_attribute_owned(Attribute::AtrString) {
            Err(error) => {return Err(error.to_string());},
            Ok(result) => result
        };
        if buffer == APOLLO_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(ApolloCardReader{})});}
        if buffer == GEMALTO_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(GemaltoCardReader{})});}

        return Err("Unknown card type".to_string());
    }
    fn fit_in(&mut self, map: &HashMap<u16, Vec<u8>>) {
        for tag in ID_TAGS.iter().enumerate() {
            PersonalIdItem::new(tag.1, map).and_then(|item: PersonalIdItem| Some(self.personal.insert(item.tag.id, item)));    
        }
    }

    pub fn read_id(&mut self, card: &Card) -> Result<(),String> {
        self.card_reader.select_aid(card, LICNA_KARTA_AID)?;

        let buffer = self.card_reader.read_raw_file(&card, PERSONAL_FILE, false)?;
        let res = self.card_reader.parse_tlv(&buffer)?;
        self.fit_in(&res);

        let buffer = self.card_reader.read_raw_file(&card, RESIDENCE_FILE, false)?;
        let res = self.card_reader.parse_tlv(&buffer)?;
        self.fit_in(&res);

        let buffer = self.card_reader.read_raw_file(&card, DOCUMENT_FILE, false)?;
        let res = self.card_reader.parse_tlv(&buffer) ?;
        self.fit_in(&res);
        
        self.image = self.card_reader.read_raw_file(&card, PHOTO_FILE, true)?;
        
        Ok(())
    }

    pub fn to_json(&self) -> String {
        let mut json_output: String = String::new();
        json_output.push_str("{");
        for (_tag, item ) in self.personal.iter() {
            json_output.push_str(&format!("\"{}\": \"{}\",\n", &item.tag.json_id, &item.value));
        }
        json_output.push_str(&format!("\"Image\": \"{}\"\n", &base64::encode(&self.image)));
        json_output.push_str("}");
        json_output
    }
}


impl fmt::Display for PersonalIdItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.tag.description, self.value)
    }
}

trait CardReader {
    fn select_aid(&self, card: &Card, aid: &[u8]) -> Result<Vec<u8>, String>;
    fn parse_tlv(&self, buffer: &Vec<u8>) -> Result<HashMap<u16, Vec<u8>>, String>;
    fn select_file(&self, card: &Card, file: &[u8], expected_result_size: u8) -> Result<Vec<u8>, String>;
    fn read_binary(&self, card: &Card, offset: u32, length: u32) -> Result<Vec<u8>, String>;
    fn read_raw_file(&self, card: &Card, file: &[u8], strip_tag: bool) -> Result<Vec<u8>, String>;
}

struct ApolloCardReader {
}

struct GemaltoCardReader {
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

impl CardReader for GemaltoCardReader  {
    fn select_aid(&self, card: &Card, aid: &[u8]) -> Result<Vec<u8>, String> {
        let apdu: &[u8] = &[0x00, 0xa4, 0x04, 0x00, aid.len() as u8];
         let apdu = [apdu, aid].concat();

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
