use jni::objects::{JClass, JObject, JString, JValue};
use jni::JNIEnv;
use jni_fn::jni_fn;
use std::{collections::HashMap, sync::Arc};
use jni::sys::jint;

use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::RENDERER;
use wgpu_mc::mc::entity::Entity;
use wgpu_mc::{
    mc::entity::{Cuboid, CuboidUV, EntityPart, PartTransform},
    render::atlas::Atlas,
};
use wgpu_mc::render::pipeline::ENTITY_ATLAS;

#[derive(Debug, Deserialize)]
pub struct ModelCuboidData {
    pub name: Option<String>,
    pub offset: HashMap<String, f32>,
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

#[derive(Debug, Copy, Clone)]
pub struct AtlasPosition {
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32,
}

impl AtlasPosition {
    pub fn map(&self, pos: [f32; 2]) -> [f32; 2] {
        [
            (self.x + pos[0]) / (self.width as f32),
            (self.y + pos[1]) / (self.height as f32),
        ]
    }
}

pub fn tmd_to_wm(name: String, part: &ModelPartData, ap: &AtlasPosition) -> Option<EntityPart> {
    Some(EntityPart {
        name,
        transform: PartTransform {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pivot_x: part.transform.pivot_x,
            pivot_y: part.transform.pivot_y,
            pivot_z: part.transform.pivot_z,
            yaw: part.transform.yaw,
            pitch: part.transform.pitch,
            roll: part.transform.roll,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        },
        cuboids: part
            .cuboid_data
            .iter()
            .map(|cuboid_data| {
                let pos = [
                    *cuboid_data.texture_uv.get("x").unwrap(),
                    *cuboid_data.texture_uv.get("y").unwrap(),
                ];
                let dimensions = [
                    cuboid_data.dimensions.get("x").unwrap(),
                    cuboid_data.dimensions.get("y").unwrap(),
                    cuboid_data.dimensions.get("z").unwrap(),
                ];

                Some(Cuboid {
                    x: *cuboid_data.offset.get("x")?,
                    y: *cuboid_data.offset.get("y")?,
                    z: *cuboid_data.offset.get("z")?,
                    width: *cuboid_data.dimensions.get("x")?,
                    height: *cuboid_data.dimensions.get("y")?,
                    length: *cuboid_data.dimensions.get("z")?,
                    textures: CuboidUV {
                        west: [
                            ap.map([pos[0], pos[1] + dimensions[2]]),
                            ap.map([
                                pos[0] + dimensions[0],
                                pos[1] + (dimensions[2] + dimensions[1]),
                            ]),
                        ],
                        east: [
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1] + dimensions[2]]),
                            ap.map([
                                (pos[0] + (dimensions[0] * 3.0)),
                                pos[1] + (dimensions[2] + dimensions[1]),
                            ]),
                        ],
                        south: [
                            ap.map([(pos[0] + (dimensions[0])), pos[1] + dimensions[2]]),
                            ap.map([
                                (pos[0] + (dimensions[0] * 2.0)),
                                pos[1] + (dimensions[2] + dimensions[1]),
                            ]),
                        ],
                        north: [
                            ap.map([(pos[0] + (dimensions[0] * 3.0)), pos[1] + dimensions[2]]),
                            ap.map([
                                (pos[0] + (dimensions[0] * 4.0)),
                                pos[1] + (dimensions[2] + dimensions[1]),
                            ]),
                        ],
                        up: [
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1]]),
                            ap.map([(pos[0] + (dimensions[0] * 3.0)), pos[1] + (dimensions[2])]),
                        ],
                        down: [
                            ap.map([(pos[0] + dimensions[0]), pos[1]]),
                            ap.map([(pos[0] + (dimensions[0] * 2.0)), pos[1] + (dimensions[2])]),
                        ],
                    },
                })
            })
            .collect::<Option<Vec<Cuboid>>>()?,
        children: part
            .children
            .iter()
            .map(|(name, part)| tmd_to_wm(name.clone(), part, ap))
            .collect::<Option<Vec<EntityPart>>>()?,
    })
}

#[derive(Deserialize)]
pub struct Wrapper2 {
    data: ModelPartData,
}

#[derive(Deserialize)]
pub struct Wrapper1 {
    data: Wrapper2,
}

static ENTITY_MPD: OnceCell<HashMap<String, ModelPartData>> = OnceCell::new();

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn registerEntities(env: JNIEnv, _class: JClass, string: JString) {
    let wm = RENDERER.get().unwrap();

    let entities_json_javastr = env.get_string(string).unwrap();
    let entities_json: String = entities_json_javastr.into();

    let mpd: HashMap<String, ModelPartData> = serde_json::from_str::<HashMap<String, Wrapper1>>(&entities_json)
        .unwrap()
        .into_iter()
        .map(|(name, wrapper)| (name, wrapper.data.data))
        .collect();

    ENTITY_MPD.set(mpd);

    let mpd = ENTITY_MPD.get().unwrap();

    let atlas_position = AtlasPosition {
        width: 0,
        height: 0,
        x: 0.0,
        y: 0.0,
    };

    let atlases = wm.mc.texture_manager.atlases.load();
    let atlas = atlases.get(ENTITY_ATLAS).unwrap();

    let entities: Vec<Arc<Entity>> = mpd
        .iter()
        .map(|(name, mpd)| {
            let entity_part = tmd_to_wm("root".into(), mpd, &atlas_position).unwrap();

            Arc::new(Entity::new(name.clone(), entity_part, &*wm.wgpu_state, atlas.load().bindable_texture.clone()))
        })
        .collect();

    entities.iter().for_each(|entity| {
        let entity_string = env.new_string(&entity.name).unwrap();

        entity.parts.iter().for_each(|(name, index)| {
            let part_string = env.new_string(name).unwrap();

            env.call_static_method(
                "dev/birb/wgpu/render/Wgpu",
                "helperSetPartIndex",
                "(Ljava/lang/String;Ljava/lang/String;I)V",
                unsafe { &[
                    JValue::Object(JObject::from_raw(entity_string.into_raw())),
                    JValue::Object(JObject::from_raw(part_string.into_raw())),
                    JValue::Int(*index as jint),
                ] },
            )
                .unwrap();
        });
    });

    *wm.mc.entity_models.write() = entities;
}
