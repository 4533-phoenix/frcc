use log::error;
use once_cell::sync::Lazy;
use std::process;
use tera::Tera;

#[cfg(feature = "include_templates")]
use include_dir::include_dir;

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    #[cfg(feature = "include_templates")]
    let mut tera = {
        let dir = include_dir!("templates");
        let mut tera = Tera::default();

        let templates = dir
            .files()
            .map(|file| {
                let name = file.path().to_str().unwrap();
                let content = std::str::from_utf8(file.contents()).unwrap();
                (name, content)
            })
            .collect::<Vec<_>>();

        match tera.add_raw_templates(templates) {
            Ok(_) => tera,
            Err(e) => {
                error!("Parsing error(s): {}", e);
                process::exit(1);
            }
        }
    };

    #[cfg(not(feature = "include_templates"))]
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };

    tera
});
