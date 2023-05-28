use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use wgpu_mc::mc::block::{BlockstateKey, ChunkBlockState};
use wgpu_mc::mc::chunk::{BlockStateProvider, Chunk, LightLevel};
use wgpu_mc::mc::MinecraftState;
use wgpu_mc::minecraft_assets::schemas::blockstates::multipart::StateValue;

use wgpu_mc::render::pipeline::BLOCK_ATLAS;
use wgpu_mc::WmRenderer;

struct SimpleBlockstateProvider(Arc<MinecraftState>, BlockstateKey);

impl BlockStateProvider for SimpleBlockstateProvider {
    fn get_state(&self, x: i32, y: i16, z: i32) -> ChunkBlockState {
        if x >= 0 && x < 16 && y >= 0 && y < 200 && z >= 0 && z < 16 {
        // if x == 0 && y == 0 && z == 0 {
            // ChunkBlockState::State(self.1)
            ChunkBlockState::State(self.1)
        } else {
            ChunkBlockState::Air
        }
    }

    fn get_light_level(&self, x: i32, y: i16, z: i32) -> LightLevel {
        LightLevel::from_sky_and_block(15, 15)
    }

    fn is_section_empty(&self, _index: usize) -> bool {
        false
    }
}

impl Debug for SimpleBlockstateProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

pub fn make_chunks(wm: &WmRenderer) -> Chunk {
    let bm = wm.mc.block_manager.read();
    let atlas = wm
        .mc
        .texture_manager
        .atlases
        .load()
        .get(BLOCK_ATLAS)
        .unwrap()
        .load();

    let (index, _, block) = bm.blocks.get_full("minecraft:furnace").unwrap();

    let (_, augment) = block
        .get_model_by_key(
            [("facing", &StateValue::String("north".into())), ("lit", &StateValue::Bool(true))],
            &*wm.mc.resource_provider,
            &atlas,
        )
        .unwrap();

    let provider = SimpleBlockstateProvider(
        wm.mc.clone(),
        BlockstateKey {
            block: index as u16,
            augment,
        },
    );

    let chunk = Chunk::new([0, 0]);
    let time = Instant::now();

    let pipelines = wm.pipelines.load();
    let layers = pipelines.chunk_layers.load();

    chunk.bake_chunk(wm, &layers, &bm, &provider);

    println!(
        "Built 1 chunk in {} microseconds",
        Instant::now().duration_since(time).as_micros()
    );

    chunk
}
