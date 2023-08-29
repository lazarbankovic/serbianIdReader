extern crate printpdf;
use printpdf::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufWriter, Write};
use idreader_lib::module_reader::reader::{PersonalId, PersonalIdTag, PersonalIdItem};

fn add_line(x: f64, y: f64, current_layer: &PdfLayerReference) {
    let points1 = vec![
    (Point::new(Mm(x), Mm(y)), false),
    (Point::new(Mm(x+166.0), Mm(y)), false)
    ];
    
    let line1 = Line {
        points: points1,
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };
    
    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));

    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);
    current_layer.add_shape(line1);

}

fn add_image(x: f64, y: f64, buffer: &[u8], current_layer: &PdfLayerReference) -> Result<(), String> {
    let points1 = vec![
        (Point::new(Mm(x + 0.0), Mm(259.0)), false),
        (Point::new(Mm(x + 42.0), Mm(259.0)), false),
        (Point::new(Mm(x + 42.0), Mm(203.0)), false),
        (Point::new(Mm(x + 0.0), Mm(203.0)), false),
    ];

    let line1 = Line {
        points: points1,
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.0, 0.0, 0.0, None));
    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(20);

    current_layer.set_fill_color(fill_color);
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(0.1);

    current_layer.add_shape(line1);


    let dyn_image = image_crate::load_from_memory(buffer).or_else(|err| return Err(err.to_string()))?;
    let ximage = ImageXObject::from_dynamic_image(&dyn_image);
    let image = Image::from(ximage);
    image.add_to_layer(
        current_layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(x)),
            translate_y: Some(Mm(y)),
            rotate: None,
            scale_x: Some(2.06),
            scale_y: Some(2.06),
            dpi: None,
        },
    );
    Ok(())
}

fn add_text(x:f64, y:f64, text: &str, font_size: f64, font: &IndirectFontRef, current_layer: &PdfLayerReference) {
    current_layer.begin_text_section();

    current_layer.set_font(&font, font_size);
    current_layer.set_text_cursor(Mm(x), Mm(y));
    current_layer.set_line_height(5.0);
    current_layer.set_word_spacing(5.0);
    current_layer.set_character_spacing(0.3);
    current_layer.set_text_rendering_mode(TextRenderingMode::Fill);
    current_layer.write_text(text.clone(), &font);
    current_layer.add_line_break();

    current_layer.end_text_section();
}

pub fn copy_font() {
    let font_bytes = include_bytes!("FreeSans.ttf");
    if !std::path::Path::new("/tmp/FreeSans.ttf").exists() {
        let mut file = File::create("/tmp/FreeSans.ttf").expect("failed to open file");
        file.write_all(font_bytes).expect("Failed to write the file");
    }

}

pub fn topdf(personal_id: &PersonalId, path: &str) -> Result<(), String>{
    let (doc, page1, layer1) =
        PdfDocument::new("Podaci licne karte", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let left_margin = 18.0;
    let data_margin = 49.0;

    let font_file = match File::open("/tmp/FreeSans.ttf") {
        Ok(file) => file,
        Err(str) => return Err(str.to_string())
    };
    let font2 = match doc.add_external_font(font_file) {
        Ok(font) => font,
        Err(str) => return Err(str.to_string())
    };


    let empty_item = PersonalIdItem::default();

    let surname = &personal_id.personal.get(&PersonalIdTag::Surname).unwrap_or(&empty_item).value;
    let name = &personal_id.personal.get(&PersonalIdTag::GivenName).unwrap_or(&empty_item).value;
    let birthdate = &personal_id.personal.get(&PersonalIdTag::DateOfBirth).unwrap_or(&empty_item).value;
    let place_of_birth = &personal_id.personal.get(&PersonalIdTag::PlaceOfBirth).unwrap_or(&empty_item).value;
    let state_of_birth = &personal_id.personal.get(&PersonalIdTag::StateOfBirth).unwrap_or(&empty_item).value;
    let community_of_birth = &personal_id.personal.get(&PersonalIdTag::CommunityOfBirth).unwrap_or(&empty_item).value;
    let parent = &personal_id.personal.get(&PersonalIdTag::ParentGivenName).unwrap_or(&empty_item).value;
    let state = &personal_id.personal.get(&PersonalIdTag::State).unwrap_or(&empty_item).value;
    let community = &personal_id.personal.get(&PersonalIdTag::Community).unwrap_or(&empty_item).value;
    let address = &personal_id.personal.get(&PersonalIdTag::Street).unwrap_or(&empty_item).value;
    let house_number = &personal_id.personal.get(&PersonalIdTag::HouseNumber).unwrap_or(&empty_item).value;
    let place = &personal_id.personal.get(&PersonalIdTag::Place).unwrap_or(&empty_item).value;
    let personal_number = &personal_id.personal.get(&PersonalIdTag::PersonalNumber).unwrap_or(&empty_item).value;
    let gender = &personal_id.personal.get(&PersonalIdTag::Sex).unwrap_or(&empty_item).value;
    let authority = &personal_id.personal.get(&PersonalIdTag::IssuingAuthority).unwrap_or(&empty_item).value;
    let id_no = &personal_id.personal.get(&PersonalIdTag::DocRegNo).unwrap_or(&empty_item).value;
    let issuing_date = &personal_id.personal.get(&PersonalIdTag::IssuingDate).unwrap_or(&empty_item).value;
    let expiry_date = &personal_id.personal.get(&PersonalIdTag::ExpiryDate).unwrap_or(&empty_item).value;

    add_line(left_margin, 277.0, &current_layer);
    add_text(left_margin+2.0, 269.0, "ЧИТАЧ ЕЛЕКТРОНСКЕ ЛИЧНЕ КАРТЕ: ШТАМПА ПОДАТАКА", 15.5, &font2, &current_layer);
    add_line(left_margin, 265.0, &current_layer);

    add_line(left_margin, 196.0, &current_layer);
    add_text(left_margin+2.0, 190.0, "Подаци о грађанину", 12.0, &font2, &current_layer);
    add_line(left_margin, 187.0, &current_layer);

    add_text(left_margin+2.0, 181.0, "Презиме:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 181.0, surname, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 173.0, "Име:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 173.0, name, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 165.0, "Име једног родитеља:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 165.0, parent, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 157.0, "Датум рођења:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 157.0, birthdate, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 148.0, "Место рођења,", 11.0, &font2, &current_layer);
    add_text(left_margin+2.0, 144.0, "општина и држава:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 146.0, &[place_of_birth,",",community_of_birth,",",state_of_birth].to_vec().concat(), 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 135.0, "Пребивалиште и", 11.0, &font2, &current_layer);
    add_text(left_margin+2.0, 131.0, "адреса стана:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 133.0, &[address,",",house_number,",",community,",", place, ",", state].to_vec().concat(), 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 121.0, "ЈМБГ:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 121.0, personal_number, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 111.0, "Пол:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 111.0, gender, 11.0, &font2, &current_layer);

    add_line(left_margin, 107.0, &current_layer);
    add_text(left_margin+2.0, 101.5, "Подаци о документу", 12.0, &font2, &current_layer);
    add_line(left_margin, 98.0, &current_layer);

    add_text(left_margin+2.0, 91.0, "Документ издаје:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 91.0, authority, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 83.0, "Број документа:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 83.0, id_no, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 75.0, "Датум издавања:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 75.0, issuing_date, 11.0, &font2, &current_layer);

    add_text(left_margin+2.0, 67.0, "Важи до:", 11.0, &font2, &current_layer);
    add_text(left_margin+data_margin, 67.0, expiry_date, 11.0, &font2, &current_layer);

    add_image(left_margin, 203.0, &personal_id.image, &current_layer).unwrap();
    let pdf_file = match File::create(&[path,"/",personal_number,".pdf"].concat()) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string())
    };

    match doc.save(&mut BufWriter::new(pdf_file)) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string())
    }
}
