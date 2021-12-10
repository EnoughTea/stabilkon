use std::{env, path};

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, KeyCode};
use ggez::graphics::{
    self, Color, DrawParam, Drawable, FilterMode, Image, Mesh, Rect, Text, Vertex,
};
use ggez::{input, timer, Context, GameError, GameResult};
use glam::*;
use rand::*;
use stabilkon::{MeshBuilder, UvFlip};

pub(crate) fn pressed_keys_to_axis(
    ctx: &Context,
    negative_key: KeyCode,
    positive_key: KeyCode,
) -> f32 {
    if input::keyboard::is_key_pressed(ctx, negative_key) {
        -1.0
    } else if input::keyboard::is_key_pressed(ctx, positive_key) {
        1.0
    } else {
        0.0
    }
}

pub(crate) fn value_or_shifted<T>(ctx: &Context, value: T, shifted_value: T) -> T {
    if input::keyboard::is_key_pressed(ctx, KeyCode::LShift)
        || input::keyboard::is_key_pressed(ctx, KeyCode::RShift)
    {
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
            inner.push("test_ggez_integration/gltests/resources");
            inner
        }
    }
}

struct GameState {
    camera_pos: Vec2,
    camera_scale: f32,
    terrain: Mesh,
    doodads: Mesh,
    total_quads: u32,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let mut rng = thread_rng();
        let mut texture_atlas = Image::new(ctx, "/forest_tiles.png")?;
        texture_atlas.set_filter(FilterMode::Nearest);
        let texture_atlas_size = [texture_atlas.width() as f32, texture_atlas.height() as f32];
        let tile_size = 32.0_f32;
        let terrain_size = [1024_i32, 1024];
        let terrain_tiles_count = (terrain_size[0] * terrain_size[1]) as u32;
        let white_color = [1.0_f32, 1.0, 1.0, 1.0];
        // Gather source rectangles for several tile images in the texture atlas:
        let plain_grass_source = [0.0, 0.0, tile_size, tile_size];
        let flowers1_source = [32.0, 0.0, tile_size, tile_size];
        let flowers2_source = [64.0, 0.0, tile_size, tile_size];
        let mut doodad_sources: Vec<[f32; 4]> = Vec::with_capacity(7);
        for i in 0..7 {
            doodad_sources.push([i as f32 * tile_size, 96.0, tile_size, tile_size])
        }

        // Create grassy plain with flowers:
        let mut terrain_mesh_builder: MeshBuilder<Vertex> =
            MeshBuilder::new(texture_atlas_size, terrain_tiles_count)
                .map_err(|e| GameError::CustomError(e.to_string()))?;
        let mut terrain_quad_index = 0_u32;
        for y in -terrain_size[1] / 2..terrain_size[1] / 2 {
            for x in -terrain_size[0] / 2..terrain_size[0] / 2 {
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
        let terrain = terrain_mesh_builder.create_mesh(ctx, texture_atlas.clone())?;

        // Create bushes and stumps to lay over the grassy terrain:
        let doodads_count = ((terrain_size[0] / 2) * (terrain_size[1] / 2)) as u32;
        let mut doodads_mesh_builder: MeshBuilder<Vertex> =
            MeshBuilder::new(texture_atlas_size, doodads_count)
                .map_err(|e| GameError::CustomError(e.to_string()))?;
        let mut doodad_quad_index = 0_u32;
        for y in -terrain_size[1] / 2..terrain_size[1] / 2 {
            for x in -terrain_size[0] / 2..terrain_size[0] / 2 {
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
        let doodads = doodads_mesh_builder.create_mesh(ctx, texture_atlas)?;

        Ok(GameState {
            camera_pos: Vec2::ZERO,
            camera_scale: 1.0,
            terrain,
            doodads,
            total_quads: terrain_tiles_count + doodads_count,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = timer::duration_to_f64(timer::delta(ctx)) as f32;
        let move_speed = value_or_shifted(ctx, 500.0, 3000.0);
        let scale_speed = value_or_shifted(ctx, 2.0, 15.0);
        let horizontal_movement = pressed_keys_to_axis(ctx, KeyCode::A, KeyCode::D);
        let vertical_movement = pressed_keys_to_axis(ctx, KeyCode::W, KeyCode::S);
        let scale_movement = pressed_keys_to_axis(ctx, KeyCode::Q, KeyCode::E);

        self.camera_pos.x += horizontal_movement * (move_speed * dt);
        self.camera_pos.y += vertical_movement * (move_speed * dt);

        let scale_change = scale_movement * (scale_speed * dt);
        self.camera_scale = (self.camera_scale + scale_change).clamp(0.25, 25.0);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let camera = Mat4::from_scale_rotation_translation(
            Vec3::splat(self.camera_scale),
            Quat::IDENTITY,
            Vec3::new(self.camera_pos.x, self.camera_pos.y, 0.0),
        );

        // Fake global MVP, because the proper method to set it is private:
        let (w, h) = graphics::drawable_size(ctx);
        let scaled_pos = camera.transform_vector3(Vec3::new(-w / 2.0, -h / 2.0, 0.0));
        let scaled_size = camera.transform_vector3(Vec3::new(w, h, 0.0));
        graphics::set_screen_coordinates(
            ctx,
            Rect::new(scaled_pos.x, scaled_pos.y, scaled_size.x, scaled_size.y),
        )?;

        // Draw terrain meshes using camera offset, since screen coords were just offseted with it:
        let origin = DrawParam::new();
        self.terrain.draw(
            ctx,
            origin.dest(Vec2::new(-self.camera_pos.x, -self.camera_pos.y)),
        )?;
        self.doodads.draw(
            ctx,
            origin.dest(Vec2::new(-self.camera_pos.x, -self.camera_pos.y)),
        )?;

        // Revert MVP hack for text drawing
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, w, h))?;
        let debug_text = Text::new(format!(
            "{} quads - {:.0} FPS",
            self.total_quads,
            timer::fps(ctx)
        ));
        graphics::draw(ctx, &debug_text, (Vec2::new(16.0, 16.0), Color::WHITE))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = get_resource_dir();
    let cb = ggez::ContextBuilder::new("ggez_integration", "none")
        .window_setup(WindowSetup {
            title: "Static sprites demo (WASD to move, QE to zoom)".to_owned(),
            samples: ggez::conf::NumSamples::One,
            vsync: false,
            icon: "".to_owned(),
            srgb: true,
        })
        .window_mode(WindowMode {
            width: 1280.0,
            height: 720.0,
            maximized: true,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            borderless: false,
            min_width: 800.0,
            min_height: 600.0,
            max_width: 0.0,
            max_height: 0.0,
            resizable: true,
            visible: true,
            resize_on_scale_factor_change: false,
        })
        .add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = GameState::new(&mut ctx)?;

    event::run(ctx, event_loop, state)
}
