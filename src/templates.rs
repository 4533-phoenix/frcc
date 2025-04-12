use log::error;
use once_cell::sync::Lazy;
use std::process;
use tera::Tera;

#[derive(RustEmbed)]
#[folder = "templates"]
struct Templates;

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();

    let templates = Templates::iter()
        .map(|fname| {
            let file = Templates::get(&fname).unwrap();
            let content = String::from_utf8(file.data.to_vec()).unwrap();
            (fname, content)
        })
        .collect::<Vec<_>>();

    match tera.add_raw_templates(templates) {
        Ok(_) => tera,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            process::exit(1);
        }
    }
});
