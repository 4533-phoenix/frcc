use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use tempfile::TempDir;

use crate::{Ability, render_back_card};

use super::render_front_card;
use serde::Serialize;

type PrintoutCfg = Vec<PrintoutCfgCard>;
#[derive(Serialize)]
struct PrintoutCfgCard {
    front: String,
    back: String,
}

pub async fn generate_printout(
    ids: Vec<String>,
    robot_name: impl Into<String>,
    team_num: impl Into<String>,
    image_path: impl Into<String>,
    abilities: impl Iterator<Item = Ability>,
    printout_id: impl Into<String>,
) -> String {
    let printout_id = printout_id.into();

    let temp = TempDir::new().unwrap().into_path();
    let printout_path = format!("printouts/{printout_id}.pdf");

    render_front_card(
        include_str!("../../../cards/front/default.svg"),
        &robot_name.into(),
        &team_num.into(),
        &image_path.into(),
        &abilities.collect::<Vec<_>>(),
        Some(temp.join("front.png").to_str().unwrap()),
    );

    let printout_config: Arc<Mutex<PrintoutCfg>> = Arc::new(Mutex::new(Vec::new()));
    ids.into_par_iter().for_each(|id| {
        render_back_card(
            include_str!("../../../cards/back/default.svg"),
            &id,
            Some(temp.join(format!("{id}.png")).to_str().unwrap()),
        );
        (*printout_config.lock().unwrap()).push(PrintoutCfgCard {
            front: String::from("front.png"),
            back: format!("{id}.png"),
        });
    });

    {
        let mut wr = File::create(temp.join("config.json")).unwrap();
        serde_json::to_writer(&mut wr, &printout_config).unwrap();
        wr.flush().unwrap();
    }

    let mut cmd = Command::new("typst")
        .args([
            "compile",
            "--root",
            temp.to_str().unwrap(),
            "-",
            &printout_path,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(ref mut stdin) = cmd.stdin {
        stdin.write_all(include_bytes!("../printout.typ")).unwrap();
    }

    cmd.wait().unwrap();

    std::fs::remove_dir_all(temp).unwrap();

    printout_id
}
