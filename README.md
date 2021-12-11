# Mesh builder for tile maps using using texture atlases &emsp; [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/stabilkon.svg
[crates.io]: https://crates.io/crates/stabilkon

This library helps you create a mesh for drawing lots of small static 2D images.

![Teaser](teaser.png?raw=true "Teaser showing a forest tiled map")

Imagine creating a 2D game with a large zoomable tile map, something like Factorio.
Usual sprite batches are tailored for dynamic sprites, their data is uploaded to GPU every frame.
Using them for a grand zoomed-out scenes with lots of static images might be too slow.
So when you need that sweet render speed at the cost of GPU memory,
it is time to create meshes for chunks of your huge map yourself. This is exactly what this library helps you to do.


## Short guide:

0. `features = [ "ggez" ]` or `features = [ "tetra" ]` should be set if you plan on using one of these.
1. Create a mesh builder, `MeshFromQuads`, with either ggez, Tetra or
your own custom vertex type with `From<PosUvColor>` implemented.
Supply size of the texture which you will use for the mesh and the mesh quad limit.
All quads will be preallocated at this point.
2. Set mesh quads to various images in any order using builder's `set` methods like `set_pos_color_source`.
3. After you are done, call `create_mesh` or, if you ignored both ggez and Tetra, `into_vertices_and_indices`.
4. Draw your mesh or vertices to screen in any way you want, it is just vertices in clockwise order.
You even control UV flip in `set` methods and can use any coordinate system you want.
Default is OpenGL-tailored left-to-right bottom-to-top system,
but for examples I flip UVs vertically, since both ggez and Tetra use top-to-bottom.


## Longer guide

Take a look at either ggez test ([ggez.rs](test_ggez_integration/gltests/ggez.rs))
or Tetra test ([tetra.rs](test_tetra_integration/gltests/ggez.rs)). As usual, before launching Tetra tests,
make sure SDL2 development libraries are available. On Windows, the quickest way is to just drop them at the crate root.
Long story short, here is a tile map creation using Tetra vertex type, ggez is almost the same:

```rust
use stabilkon::*;
use tetra::{
    graphics::{
        mesh::{IndexBuffer, Mesh, Vertex, VertexBuffer},
        Color, Rectangle, Texture,
    },
    math::Vec2,
    Context, TetraError,
};
// Load texture atlas with tile images:
let tiles_texture_atlas = Texture::new(ctx, "./path/to/texture_atlas.png")?;
// We won't be using custom shaders and such - let the mesh builder fix UVs for us:
let use_half_pixel_offset = true;

// Single tile is 32×32:
let tile_size = 32.0_f32;
// Let's make test map 256×256:
let map_size = Vec2::from(256);
// Calculate required quad limit for the map:
let quad_count = map_size.x * map_size.y;
// Pick grass tile image from atlas; it is located at the very top-left of the texture atlas.
let grass_tile_source = [0.0, 0.0, 32.0, 32.0];
// Standard white color, means tile images will be drawn as-is.
let white_color = [1.0_f32, 1.0, 1.0, 1.0];
// When adding a quad to a mesh builder, you can control UV flipping with `UvFlip` parameter.
// By default the usual left-to-right, bottom-to-top system is used.
// But we decided to use left-to-right, top-to-bottom coordinate system in tile source rectangle above, so when
// adding quads using `grass_tile_source` a `UvFlip::Vertical` will be supplied.

// Create a mesh builder for an indexed mesh capable of holding entire map...
let mut terrain_mesh_builder: MeshFromQuads<Vertex> = MeshFromQuads::new(
    [
        tiles_texture_atlas.width() as f32,
        tiles_texture_atlas.height() as f32,
    ],
    use_half_pixel_offset,
    quad_count,
)?;
// ... and add a lot of quads with grass tile texture region:
let mut quad_index = 0_u32;
for y in 0..map_size.y {
    for x in 0..map_size.x {
        let position = [x as f32 * tile_size, y as f32 * tile_size];
        terrain_mesh_builder.set_pos_color_source(
            quad_index,
            position,
            white_color,
            grass_tile_source,
            UvFlip::Vertical,
        );
        quad_index += 1;
    }
}
// Finally, create a mesh consisting of quads covered with grass tile texture region:
let (terrain_mesh, terrain_vb) = terrain_mesh_builder.create_mesh(ctx, texture_atlas)?;
// All done, now you can use this mesh as usual!
```

## Update quads in a static mesh after its creation

Let's use Tetra for quad update examples. ggez is almost the same,
except you change vertex buffer directly on created mesh. 

As you can notice, `create_mesh` call in previous example returned not just a mesh, but its vertex buffer as well.
In order to change quad vertices, you need to call vertex buffer's `set_data` method
with changed vertices and their offset.


### Change a single quad

```rust
// Assume we have created indexed mesh before, it is `terrain_vb` from the upper example.
let use_indices = true;
// ...and we want to change the eight quad we have added into a hole tile.
let new_quad_index = 7;
let hole_tile_source = [160.0, 0.0, 32.0, 32.0];
// Calculate its vertex offset:
let offset = new_quad_index * vertices_per_quad(use_indices);
// Get new quad vertices:
let new_quad_params =
    PosColorSource::new([512.0, 128.0], white_color, hole_tile_source, UvFlip::Vertical);
let new_quad_vertices = new_quad_params.to_vertices(texture_size, use_indices);
// Alright, now upload new vertices at the changed offset:
terrain_vb.set_data(ctx, &new_quad_vertices, offset as usize);
```

### Change multiple quads

```rust
// Again, we will be changing `terrain_vb`, but this time we need to have access to entire vertex buffer data.
// Because we are changing random quads, we need to get proper vertices for all quads between the first
// (with the smallest index) and the last (with the biggest index) changed quads. 
// And for that we need to keep `terrain_mesh_builder` around.
// Alright, these are the quads we are going to change into hole tiles:
let changed_quads = [7, 17, 13, 11];
let hole_tile_source = [160.0, 0.0, 32.0, 32.0];
// Find the first quad and its vertex offset:
let first_changed_quad = changed_quads.iter().min().unwrap();
let first_changed_quad_vertex_offset =
    (first_changed_quad * terrain_mesh_builder.vertices_per_quad()) as usize;
// Find the last quad and its 'end' vertex offset:
let last_changed_quad = changed_quads.iter().max().unwrap();
let last_changed_quad_vertex_offset =
    (last_changed_quad * terrain_mesh_builder.vertices_per_quad()) as usize;
let after_last_changed_quad_vertex_offset =
    last_changed_quad_vertex_offset + terrain_mesh_builder.vertices_per_quad() as usize;
// Perform our changes:
for changed_quad_index in changed_quads {
    terrain_mesh_builder.set_pos_color_source(
        changed_quad_index,
        // Just extrude tiles diagonally from the map for demo purposes:
        [-(changed_quad_index as f32 * 32.0), -(changed_quad_index as f32 * 32.0)],
        white_color,
        hole_tile_source,
        UvFlip::Vertical,
    );
}
// Upload our changes to terrain vertex buffer:
let vertices_to_upload = &terrain_mesh_builder.vertices()
    [first_changed_quad_vertex_offset..after_last_changed_quad_vertex_offset];
terrain_vb.set_data(ctx, vertices_to_upload, first_changed_quad_vertex_offset);
```


## Limitations

There are 3 things you might want to keep in mind:

1. Static image builder assumes 1 texture per mesh, so all your images should be packed into a texture atlas.
If a single atlas is not enough, create several meshes and overlay them.
2. Make sure that your vertex type is plain: it should only contain values and should not contain pointers.
3. Mesh itself should be as big as possible, but not too big for GPU to handle. When in doubt, aim for 32 MiB chunks.