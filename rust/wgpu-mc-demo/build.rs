use std::collections::HashMap;
use std::env;
use std::io;
use std::path::PathBuf;

use serde::Deserialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;

    let paths_to_copy: Vec<&str> = vec!["./res/"];
    fs_extra::copy_items(&paths_to_copy, out_dir, &copy_options)?;

    let resources_root: PathBuf = "./res/assets".into();
    let resources_root_tmp: PathBuf = "./res/assets_tmp".into();
    let temp_path = PathBuf::from("./tmp/mc-jar-cache");

    if !resources_root.is_dir() {
        println!("building assets, because {resources_root:?} does not exist yet");
        fs_extra::dir::create_all(&resources_root_tmp, true)?;

        if !temp_path.is_dir() {
            fs_extra::dir::create_all(&temp_path, true)
                .expect("tmp dir for downloading client.jar data");
        }
        println!("download offical mc client data");
        let response = reqwest::blocking::get("https://launcher.mojang.com/v1/objects/37fd3c903861eeff3bc24b71eed48f828b5269c8/client.jar").unwrap();
        let content = io::Cursor::new(response.bytes()?);
        println!("unpacking offical mc client data");
        let mut zip = zip::read::ZipArchive::new(content)?;
        zip.extract(&temp_path)?;

        let mut copy_content_only = fs_extra::dir::CopyOptions::new();
        copy_content_only.content_only = true;
        copy_content_only.overwrite = true;

        fs_extra::dir::copy(
            temp_path.join("assets"),
            &resources_root_tmp,
            &copy_content_only,
        )?;
        std::fs::remove_dir_all(temp_path)?;

        fs_extra::dir::move_dir(&resources_root_tmp, &resources_root, &copy_content_only)?;
    }

    println!("copy shader source code");
    fs_extra::dir::copy("./res/wgpu_mc", &resources_root, &copy_options)?;

    // let dumped_entities = include_str!("./dumped_entities.json");
    // let dumped_e: Vec<Wrapper> = serde_json::from_str(dumped_entities).unwrap();
    
    // panic!("{:#?}", dumped_e);

    Ok(())
}
#[derive(Debug, Deserialize)]

struct Wrapper  {
    data: ModelCuboidData
}
#[derive(Debug, Deserialize)]
pub struct ModelCuboidData {
    pub name: Option<String>,
    pub offset: Option<HashMap<String, f32>>,
    pub dimensions: HashMap<String, f32>,
    pub mirror: bool,
    #[serde(rename(deserialize = "textureUV"))]
    pub texture_uv: HashMap<String, f32>,
    #[serde(rename(deserialize = "textureScale"))]
    pub texture_scale: HashMap<String, f32>,
}

#[derive(Debug, Deserialize)]
pub struct ModelTransform {
    #[serde(rename(deserialize = "pivotX"))]
    pub pivot_x: f32,
    #[serde(rename(deserialize = "pivotY"))]
    pub pivot_y: f32,
    #[serde(rename(deserialize = "pivotZ"))]
    pub pivot_z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Debug, Deserialize)]
pub struct ModelPartData {
    #[serde(rename(deserialize = "cuboidData"))]
    pub cuboid_data: Vec<ModelCuboidData>,
    #[serde(rename(deserialize = "rotationData"))]
    pub transform: ModelTransform,
    pub children: HashMap<String, ModelPartData>,
}

#[derive(Debug, Deserialize)]
pub struct ModelData {
    pub data: ModelPartData,
}

#[derive(Debug, Deserialize)]
pub struct TextureDimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct TexturedModelData {
    pub data: ModelData,
    pub dimensions: TextureDimensions,
}

// pub fn tmd_to_wm(part: &ModelPartData) -> Option<EntityPart> {
//     Some(EntityPart {
//         name: Arc::new("".into()),
//         transform: PartTransform {
//             x: 0.0,
//             y: 0.0,
//             z: 0.0,
//             pivot_x: part.transform.pivot_x,
//             pivot_y: part.transform.pivot_y,
//             pivot_z: part.transform.pivot_z,
//             yaw: part.transform.yaw,
//             pitch: part.transform.pitch,
//             roll: part.transform.roll,
//             scale_x: 1.0,
//             scale_y: 1.0,
//             scale_z: 1.0,
//         },
//         cuboids: part
//             .cuboid_data
//             .iter()
//             .map(|cuboid_data| {
//                 Some(Cuboid {
//                     x: *cuboid_data.offset.get("x")?,
//                     y: *cuboid_data.offset.get("y")?,
//                     z: *cuboid_data.offset.get("z")?,
//                     width: *cuboid_data.dimensions.get("x")?,
//                     height: *cuboid_data.dimensions.get("y")?,
//                     length: *cuboid_data.dimensions.get("z")?,
//                     textures: CuboidUV {
//                         //TODO
//                         north: ((0.0, 0.0), (0.0, 0.0)),
//                         east: ((0.0, 0.0), (0.0, 0.0)),
//                         south: ((0.0, 0.0), (0.0, 0.0)),
//                         west: ((0.0, 0.0), (0.0, 0.0)),
//                         up: ((0.0, 0.0), (0.0, 0.0)),
//                         down: ((0.0, 0.0), (0.0, 0.0)),
//                     },
//                 })
//             })
//             .collect::<Option<Vec<Cuboid>>>()?,
//         children: part
//             .children
//             .values()
//             .map(tmd_to_wm)
//             .collect::<Option<Vec<EntityPart>>>()?,
//     })
// }
