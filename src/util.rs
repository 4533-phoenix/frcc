use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
};

use image::ImageFormat;
use tmf::{
    NormalPrecisionMode, TMFMesh, TMFPrecisionInfo, TangentPrecisionMode, UvPrecisionMode,
    VertexPrecisionMode,
};

pub async fn optimize_and_save_image(id: String, obj_buf: Vec<u8>) {
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&obj_buf).unwrap();

        img.save(format!("images/{id}.png")).unwrap();
    })
    .await
    .unwrap();
}
pub async fn optimize_and_save_model(id: String, obj_buf: Vec<u8>) {
    tokio::task::spawn_blocking(move || {
        let mut obj_buf = Cursor::new(obj_buf);

        let mesh = TMFMesh::read_from_obj_one(&mut obj_buf).unwrap();

        let prec_info = TMFPrecisionInfo {
            normal_precision: NormalPrecisionMode::from_deg_dev(2.0),
            vertex_precision: VertexPrecisionMode(0.2),
            uv_prec: UvPrecisionMode::default(),
            tangent_prec: TangentPrecisionMode::from_deg_dev(2.0),
        };

        mesh.0
            .write_tmf_one(
                &mut File::create(format!("models/{id}.tmf")).unwrap(),
                &prec_info,
                id,
            )
            .unwrap();
    })
    .await
    .unwrap();
}
