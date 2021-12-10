use std::{env, path};

use rand::*;
use stabilkon::*;
use tetra::{
    graphics::{
        self,
        mesh::{Mesh, Vertex},
        text::{Font, Text},
        Camera, Color, DrawParams, FilterMode, Texture,
    },
    input::{self, Key},
    math::Vec2,
    time, window, Context, ContextBuilder, Event, Result, State, TetraError,
};

pub(crate) fn pressed_keys_to_axis(ctx: &Context, negative_key: Key, positive_key: Key) -> f32 {
    if input::is_key_down(ctx, negative_key) {
        -1.0
    } else if input::is_key_down(ctx, positive_key) {
        1.0
    } else {
        0.0
    }
}

pub(crate) fn value_or_shifted<T>(ctx: &Context, value: T, shifted_value: T) -> T {
    if input::is_key_down(ctx, Key::LeftShift) || input::is_key_down(ctx, Key::RightShift) {
        shifted_value
    } else {
        value
    }
}

pub(crate) fn get_resource_dir() -> path::PathBuf {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("gltests/resources");
        path
    } else {
        let mut local = env::current_dir().unwrap();
        local.push("gltests/resources");
        if local.exists() {
            local
        } else {
            let mut inner = env::current_dir().unwrap();
            inner.push("test_tetra_integration/gltests/resources");
            inner
        }
    }
}

struct GameState {
    debug_text: Text,
    camera: Camera,
    terrain: Mesh,
    doodads: Mesh,
    total_quads: u32,
}

impl GameState {
    fn new(ctx: &mut Context) -> Result<GameState> {
        let mut rng = thread_rng();
        let resouce_dir = get_resource_dir();
        let mut texture_atlas = Texture::new(ctx, resouce_dir.join("forest_tiles.png"))?;
        texture_atlas.set_filter_mode(ctx, FilterMode::Nearest);
        let use_half_pixel_offset = true;

        let texture_atlas_size = [texture_atlas.width() as f32, texture_atlas.height() as f32];
        let tile_size = 32.0_f32;
        let terrain_size = Vec2::from(1024_i32);
        let terrain_tiles_count = (terrain_size.x * terrain_size.y) as u32;
        let white_color = [1.0_f32, 1.0, 1.0, 1.0];

        // Gather source rectangles for several tile images in the texture atlas:
        let plain_grass_source = [0.0, 0.0, tile_size, tile_size];
        let flowers1_source = [0.0, 32.0, tile_size, tile_size];
        let flowers2_source = [0.0, 64.0, tile_size, tile_size];
        let mut doodad_sources: Vec<[f32; 4]> = Vec::with_capacity(7);
        for i in 0..7 {
            doodad_sources.push([i as f32 * tile_size, 96.0, tile_size, tile_size])
        }

        // Create grassy plain with flowers:
        let mut terrain_mesh_builder: MeshFromQuads<Vertex> = MeshFromQuads::new(
            texture_atlas_size,
            use_half_pixel_offset,
            terrain_tiles_count,
        )
        .map_err(|e| TetraError::PlatformError(e.to_string()))?;
        let mut terrain_quad_index = 0_u32;
        for y in -terrain_size.y / 2..terrain_size.y / 2 {
            for x in -terrain_size.x / 2..terrain_size.x / 2 {
                let position = [x as f32 * tile_size, y as f32 * tile_size];
                // For terrain, place 80 % of grass tiles and 20 % of flower tiles:
                let tile_kind = rng.gen_range(0..10);
                let source = match tile_kind {
                    n if n <= 7 => plain_grass_source,
                    9 => flowers1_source,
                    _ => flowers2_source,
                };
                terrain_mesh_builder.set_pos_color_source(
                    terrain_quad_index,
                    position,
                    white_color,
                    source,
                    UvFlip::Vertical,
                );
                terrain_quad_index += 1;
            }
        }
        let (terrain, _) = terrain_mesh_builder.create_mesh(ctx, texture_atlas.clone())?;

        // Create bushes and stumps to lay over the grassy terrain:
        let doodads_count = ((terrain_size.x / 2) * (terrain_size.y / 2)) as u32;
        let mut doodads_mesh_builder: MeshFromQuads<Vertex> =
            MeshFromQuads::new(texture_atlas_size, use_half_pixel_offset, doodads_count)
                .map_err(|e| TetraError::PlatformError(e.to_string()))?;
        let mut doodad_quad_index = 0_u32;
        for y in -terrain_size.y / 2..terrain_size.y / 2 {
            for x in -terrain_size.x / 2..terrain_size.x / 2 {
                let position = [x as f32 * tile_size, y as f32 * tile_size];
                // Place roughly 1 random doodad for every 4 terrain tiles.
                // Since terrain map is larger than doodad map, don't add too much doodads:
                if rng.gen_range(0..4) == 0 && doodad_quad_index < doodads_count {
                    let doodad_kind = rng.gen_range(0..7);
                    let source = doodad_sources[doodad_kind];
                    doodads_mesh_builder.set_pos_color_source(
                        doodad_quad_index,
                        position,
                        white_color,
                        source,
                        UvFlip::Vertical,
                    );
                    doodad_quad_index += 1;
                }
            }
        }
        let (doodads, _doodads_vb) = doodads_mesh_builder.create_mesh(ctx, texture_atlas)?;

        let camera = Camera::with_window_size(ctx);
        let font = Font::bmfont(ctx, resouce_dir.join("DejaVuSansMono.fnt"))?;
        let debug_text = Text::new("", font);

        window::maximize(ctx);

        Ok(GameState {
            camera,
            terrain,
            doodads,
            total_quads: terrain_tiles_count + doodads_count,
            debug_text,
        })
    }
}

impl State for GameState {
    fn event(&mut self, _: &mut Context, event: Event) -> Result<()> {
        if let Event::Resized { width, height } = event {
            self.camera.set_viewport_size(width as f32, height as f32);
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result {
        let dt = time::get_delta_time(ctx);
        let move_speed = value_or_shifted(ctx, 500.0, 3000.0);
        let scale_speed = value_or_shifted(ctx, 0.5, 3.0);
        let horizontal_movement = pressed_keys_to_axis(ctx, Key::A, Key::D);
        let vertical_movement = pressed_keys_to_axis(ctx, Key::W, Key::S);
        let scale_movement = pressed_keys_to_axis(ctx, Key::E, Key::Q);

        self.camera.position.x += horizontal_movement * (move_speed * dt.as_secs_f32());
        self.camera.position.y += vertical_movement * (move_speed * dt.as_secs_f32());

        let scale_change = scale_movement * (scale_speed * dt.as_secs_f32());
        let new_scale = (self.camera.scale.x + scale_change).clamp(0.025, 4.0);
        self.camera.scale.x = new_scale;
        self.camera.scale.y = new_scale;

        self.camera.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<()> {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        let camera_matrix = self.camera.as_matrix();
        graphics::set_transform_matrix(ctx, camera_matrix);

        self.terrain.draw(ctx, DrawParams::new());
        self.doodads.draw(ctx, DrawParams::new());

        graphics::reset_transform_matrix(ctx);

        self.debug_text.set_content(format!(
            "{} quads - {:.0} FPS",
            self.total_quads,
            time::get_fps(ctx),
        ));
        self.debug_text.draw(ctx, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

pub fn main() -> Result<()> {
    let mut ctx = ContextBuilder::new("Static sprites demo (WASD to move, QE to zoom)", 1280, 720)
        .resizable(true)
        .show_mouse(true)
        .vsync(false)
        .build()?;
    ctx.run(|ctx| GameState::new(ctx))
}
