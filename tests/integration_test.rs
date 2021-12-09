use rand::*;
use stabilkon::*;
use tetra::{
    graphics::{
        self,
        mesh::Mesh,
        text::{Font, Text},
        Camera, Color, DrawParams, Rectangle, Texture,
    },
    input::{self, Key},
    math::Vec2,
    time, window, Context, ContextBuilder, Event, State,
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

struct GameState {
    debug_text: Text,
    camera: Camera,
    terrain: Mesh,
    doodads: Mesh,
    total_quads: u32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut rng = thread_rng();
        let texture_atlas = Texture::new(ctx, "./tests/resources/forest_tiles.png")?;

        let tile_size = 32.0_f32;
        let terrain_size = Vec2::from(1024_i32);
        let terrain_tiles_count = (terrain_size.x * terrain_size.y) as u32;

        // Gather source rectangles for several tile images in the texture atlas:
        let plain_grass_source = Rectangle::new(0.0, 0.0, tile_size, tile_size);
        let flowers1_source = Rectangle::new(32.0, 0.0, tile_size, tile_size);
        let flowers2_source = Rectangle::new(64.0, 0.0, tile_size, tile_size);
        let mut doodad_sources: Vec<Rectangle> = Vec::with_capacity(7);
        for i in 0..7 {
            doodad_sources.push(Rectangle::new(
                i as f32 * tile_size,
                96.0,
                tile_size,
                tile_size,
            ))
        }

        // Create grassy plain with flowers:
        let mut terrain_mesh_builder =
            MeshBuilder::new(texture_atlas.clone(), terrain_tiles_count)?;
        let mut terrain_quad_index = 0_u32;
        for y in -terrain_size.y / 2..terrain_size.y / 2 {
            for x in -terrain_size.x / 2..terrain_size.x / 2 {
                let position = Vec2::new(x as f32, y as f32) * tile_size;
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
                    Color::WHITE,
                    source,
                    UvFlip::Vertical,
                );
                terrain_quad_index += 1;
            }
        }
        let (terrain, _terrain_vb) = terrain_mesh_builder.create_mesh(ctx)?;

        // Create bushes and stumps to lay over the grassy terrain:
        let doodads_count = ((terrain_size.x / 2) * (terrain_size.y / 2)) as u32;
        let mut doodads_mesh_builder = MeshBuilder::new(texture_atlas, doodads_count)?;
        let mut doodad_quad_index = 0_u32;
        for y in -terrain_size.y / 2..terrain_size.y / 2 {
            for x in -terrain_size.x / 2..terrain_size.x / 2 {
                let position = Vec2::new(x as f32, y as f32) * tile_size;
                // Place roughly 1 random doodad for every 4 terrain tiles.
                // Since terrain map is larger than doodad map, don't add too much doodads:
                if rng.gen_range(0..4) == 0 && doodad_quad_index < doodads_count {
                    let doodad_kind = rng.gen_range(0..7);
                    let source = doodad_sources[doodad_kind];
                    doodads_mesh_builder.set_pos_color_source(
                        doodad_quad_index,
                        position,
                        Color::WHITE,
                        source,
                        UvFlip::Vertical,
                    );
                    doodad_quad_index += 1;
                }
            }
        }
        let (doodads, _doodads_vb) = doodads_mesh_builder.create_mesh(ctx)?;

        let camera = Camera::with_window_size(ctx);
        let font = Font::bmfont(ctx, "./tests/resources/DejaVuSansMono.fnt")?;
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
    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.camera.set_viewport_size(width as f32, height as f32);
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
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

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        graphics::set_transform_matrix(ctx, self.camera.as_matrix());

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

#[test]
pub fn demo() -> tetra::Result<()> {
    let mut ctx = ContextBuilder::new("Static sprites demo (WASD to move, QE to zoom)", 1280, 720)
        .resizable(true)
        .show_mouse(true)
        .vsync(false)
        .build()?;
    ctx.run(|ctx| GameState::new(ctx))
}
