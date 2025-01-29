use byteorder::{LittleEndian, ReadBytesExt};
use std::{collections::HashMap, fmt};
use pcsc::*;
use super::{gemalto_card_reader::*, apollo_card_reader::*};

pub const BLOCK_SIZE: u32 = 254;
pub const PERSONAL_FILE: &'static [u8] = &[0x0F, 0x03];
pub const DOCUMENT_FILE: &'static [u8] = &[0x0F, 0x02];
pub const RESIDENCE_FILE: &'static [u8] = &[0x0F, 0x04];
pub const PHOTO_FILE: &'static [u8] = &[0x0F, 0x06];
pub const SUCCESS_RESPONSE: &'static [u8]= &[0x90, 0x00];

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

impl Default for PersonalIdTagType {
    fn default() -> Self {
        Self { id: PersonalIdTag::None, description: "", json_id: ""}
    }
}

pub const ID_TAG_NONE: &'static PersonalIdTagType = &PersonalIdTagType{id: PersonalIdTag::None, description: "", json_id: ""};

pub const ID_TAGS: &'static [PersonalIdTagType] = 
        &[PersonalIdTagType{id: PersonalIdTag::DocRegNo, description: "Broj dokumenta", json_id: "DocRegNo"},
          PersonalIdTagType{id: PersonalIdTag::IssuingDate, description: "Datum izdavanja", json_id: "IssuingDate"},
          PersonalIdTagType{id: PersonalIdTag::ExpiryDate, description: "Datum isteka", json_id: "ExpiryDate"},
          PersonalIdTagType{id: PersonalIdTag::IssuingAuthority, description: "Izdato od", json_id: "IssuingAuthority"},
          PersonalIdTagType{id: PersonalIdTag::PersonalNumber, description: "JMBG", json_id: "PersonalNumber"},
          PersonalIdTagType{id: PersonalIdTag::Surname, description: "Prezime", json_id: "Surname"},
          PersonalIdTagType{id: PersonalIdTag::GivenName, description: "Ime", json_id: "GivenName"},
          PersonalIdTagType{id: PersonalIdTag::ParentGivenName, description: "Ime roditelja", json_id: "ParentGivenName"},
          PersonalIdTagType{id: PersonalIdTag::Sex, description: "Pol", json_id: "Sex"},
          PersonalIdTagType{id: PersonalIdTag::PlaceOfBirth, description: "Mesto rođenja", json_id: "PlaceOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::CommunityOfBirth, description: "Opština rođenja", json_id: "CommunityOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::StateOfBirth, description: "Država rođenja", json_id: "StateOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::DateOfBirth, description: "Datum rođenja", json_id: "DateOfBirth"},
          PersonalIdTagType{id: PersonalIdTag::State, description: "Država", json_id: "State"},
          PersonalIdTagType{id: PersonalIdTag::Community, description: "Opština", json_id: "Community"},
          PersonalIdTagType{id: PersonalIdTag::Place, description: "Mesto", json_id: "Place"},
          PersonalIdTagType{id: PersonalIdTag::Street, description: "Ulica", json_id: "Street"},
          PersonalIdTagType{id: PersonalIdTag::HouseNumber, description: "Kućni broj", json_id: "HouseNumber"},
          PersonalIdTagType{id: PersonalIdTag::HouseLetter, description: "Kućna oznaka", json_id: "HouseLetter"},
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

impl Default for PersonalIdItem {
    fn default() -> Self {
        Self { tag: ID_TAG_NONE, value: String::default() }
    }
}


impl PersonalIdItem {
    pub fn new(tag: &'static PersonalIdTagType, map: &HashMap<u16, Vec<u8>>) -> Option<PersonalIdItem> {
        let val = map.get(&(tag.id as u16))?;
        let s = match std::str::from_utf8(&val) {
            Ok(v) => v,
            Err(_) => {return Option::None}//panic!("Invalid UTF-8 sequence: {}", e),
        };
        Some(PersonalIdItem {tag: tag, value: s.to_string()})
    }
}

impl fmt::Display for PersonalIdItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >20}: {}", self.tag.description, self.value)
    }
}

pub struct PersonalId {
    pub personal: HashMap<PersonalIdTag, PersonalIdItem>,
    pub image: Vec<u8>,
    pub card_reader: Box<dyn CardReader>,
}

pub trait CardReader {
    fn select_aid(&self, card: &Card) -> Result<Vec<u8>, String>;
    fn select_file(&self, card: &Card, file: &[u8], expected_result_size: u8) -> Result<Vec<u8>, String>;
    fn read_binary(&self, card: &Card, offset: u32, length: u32) -> Result<Vec<u8>, String>;
    fn read_raw_file(&self, card: &Card, file: &[u8], strip_tag: bool) -> Result<Vec<u8>, String>;
}

impl PersonalId {
    pub fn new(card: &Card) -> core::result::Result<PersonalId, String> {
        let buffer = match card.get_attribute_owned(Attribute::AtrString) {
            Err(error) => {return Err(error.to_string());},
            Ok(result) => result
        };
        if buffer == APOLLO_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(ApolloCardReader{})});}
        if buffer == GEMALTO_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(GemaltoCardReader{})});}
        if buffer == GEMALTO_NEW_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(GemaltoCardReader{})});}
        if buffer == GEMALTO_EVEN_NEWER_CARD_TYPE { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(GemaltoCardReader{})});}
        if buffer == GEMALTO_CARD_TYPE_1 { return Ok(PersonalId { personal: HashMap::new(), image: vec![], card_reader: Box::new(GemaltoCardReader{})});}
        return Err("Unknown card type".to_string());
    }
    fn fit_in(&mut self, map: &HashMap<u16, Vec<u8>>) {
        for tag in ID_TAGS.iter().enumerate() {
            PersonalIdItem::new(tag.1, map).and_then(|item: PersonalIdItem| Some(self.personal.insert(item.tag.id, item)));    
        }
    }

    fn parse_tlv(buffer: &Vec<u8>) -> Result<HashMap<u16, Vec<u8>>, String> {
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

    pub fn read_id(&mut self, card: &Card) -> Result<(),String> {
        self.card_reader.select_aid(card)?;

        let buffer = self.card_reader.read_raw_file(&card, PERSONAL_FILE, false)?;
        let res = Self::parse_tlv(&buffer)?;
        self.fit_in(&res);

        let buffer = self.card_reader.read_raw_file(&card, RESIDENCE_FILE, false)?;
        let res = Self::parse_tlv(&buffer)?;
        self.fit_in(&res);

        let buffer = self.card_reader.read_raw_file(&card, DOCUMENT_FILE, false)?;
        let res = Self::parse_tlv(&buffer) ?;
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
