use std::fs;
use pcsc::*;
mod idreader;
use viuer::Config;
mod pdf;
use clap::Parser;

/// Serbian IDCard reader
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Dump to pdf on path 
    #[clap(short = 'p', long, value_name = "PATH", value_hint = clap::ValueHint::DirPath )]
    to_pdf: Option<String>,

    /// Dump to JSON to dir path
    #[clap(short = 'j', long, value_name = "PATH", value_hint = clap::ValueHint::DirPath )]
    to_json: Option<String>,

    /// Dump to JSON to stdout
    #[clap(short = 'o', long, action)]
    to_json_stdout: bool,
}

fn main() {
    let args = Args::parse();

    // Establish a PC/SC context.
    let ctx = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to establish context: {}", err);
            std::process::exit(1);
        }
    };

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {}", err);
            std::process::exit(1);
        }
    };

    // Use the first reader.
    let reader = match readers.next() {
        Some(reader) => reader,
        None => {
            println!("No readers are connected.");
            return;
        }
    };

    // Connect to the card.
    let result = ctx.connect(reader, ShareMode::Shared, Protocols::ANY);
    let card = match result {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("A smartcard is not present in the reader.");
            return;
        }
        Err(err) => {
            eprintln!("Failed to connect to card: {}", err);
            std::process::exit(1);
        }
    };

    let mut personal_id = idreader::PersonalId::new(&card).unwrap();
    personal_id.read_id(&card).unwrap();


    if args.to_pdf == None && args.to_json == None && !args.to_json_stdout {
        let conf = Config { absolute_offset:false, x: 0, y: 0, width: Some(42), height: Some(28), ..Default::default()};
        let img = image::load_from_memory(&personal_id.image).expect("Could not be read");

        viuer::print(&img, &conf).expect("Image printing failed.");
        for (_tag, item) in personal_id.personal.iter() {
            println!("{}", item);
        }
    }

    if let Some(path) = args.to_json {
        if !path.is_empty() {
            if let Some(personal_number) = personal_id.personal.get(&crate::idreader::PersonalIdTag::PersonalNumber) {
                fs::write(&[path, personal_number.value.clone(), ".json".to_string()].concat(), personal_id.to_json()).expect("Unable to write file");
            }
        }
    }

    if args.to_json_stdout {
        println!("{}", personal_id.to_json());
    }

    if let Some(path) = args.to_pdf {
        if !path.is_empty() {
            pdf::topdf(&personal_id, &path).unwrap();            
        }
    }

}
