use orcs_game::app::{self, Options};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 {
        match args[0].as_str() {
            "-h" | "--help" => {
                println!("{}", app::USAGE);
                return;
            }
            "-V" | "--version" => {
                println!("orcs-from-the-neighborhood {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--check-assets" => {
                let root = orcs_game::persistence::project_root();
                let db = orcs_core::CardDb::shared();
                let missing: Vec<_> = db
                    .iter()
                    .map(|card| card.image.as_str())
                    .chain(std::iter::once(orcs_game::assets::CARD_BACK))
                    .filter(|image| !root.join(image).is_file())
                    .collect();
                if !missing.is_empty() {
                    eprintln!("Missing artwork: {}", missing.join(", "));
                    std::process::exit(1);
                }
                println!("Artwork OK: 90 cards and card back");
                return;
            }
            _ => {}
        }
    }
    let options = match Options::from_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    app::build(options).run();
}
