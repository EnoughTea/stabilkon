#[cfg(all(feature = "ggez", feature = "tetra"))]
compile_error!("Only singular feature \"ggez\" or \"tetra\" must be enabled for this crate");

mod common_types;
mod draw_params;

#[cfg(not(any(feature = "ggez", feature = "tetra")))]
pub use common_types::*;
pub use draw_params::*;
pub use mint;

use snafu::{ensure, Backtrace, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(all(feature = "ggez", not(feature = "tetra")))]
type Color = ggez::graphics::Color;
#[cfg(all(feature = "ggez", not(feature = "tetra")))]
type Rectangle = ggez::graphics::Rect;
#[cfg(all(feature = "ggez", not(feature = "tetra")))]
type Vec2 = ggez::mint::Vector2<f32>;
#[cfg(all(feature = "ggez", not(feature = "tetra")))]
type Vertex = ggez::graphics::Vertex;

#[cfg(all(feature = "tetra", not(feature = "ggez")))]
type Color = tetra::graphics::Color;
#[cfg(all(feature = "tetra", not(feature = "ggez")))]
type Rectangle = tetra::graphics::Rectangle<f32>;
#[cfg(all(feature = "tetra", not(feature = "ggez")))]
type Vec2 = tetra::math::Vec2<f32>;
#[cfg(all(feature = "tetra", not(feature = "ggez")))]
type Vertex = tetra::graphics::mesh::Vertex;

#[cfg(not(any(feature = "ggez", feature = "tetra")))]
type Color = crate::common_types::Color;
#[cfg(not(any(feature = "ggez", feature = "tetra")))]
type Rectangle = crate::common_types::Rectangle;
#[cfg(not(any(feature = "ggez", feature = "tetra")))]
type Vec2 = crate::common_types::Vec2;
#[cfg(not(any(feature = "ggez", feature = "tetra")))]
type Vertex = crate::common_types::Vertex;

/// OpenGL does not specify max size for buffers, so it is driver-dependent.
/// As of 2021, high-end GPU can be expected to allocate 512 MB for a single vertex buffer.
/// Certain Intel drivers would fail for anything over 32 MB though.
pub const MAX_VERTEX_BUFFER_SIZE_MBYTES: f32 = 512.0;

#[cfg(feature = "ggez")]
static VERTEX_ZERO: Vertex = Vertex {
    color: [0.0, 0.0, 0.0, 0.0],
    pos: [0.0, 0.0],
    uv: [0.0, 0.0],
};
#[cfg(any(feature = "tetra", not(feature = "ggez")))]
static VERTEX_ZERO: Vertex = Vertex {
    position: Vec2 { x: 0.0, y: 0.0 },
    uv: Vec2 { x: 0.0, y: 0.0 },
    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
};

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Quad count is too large"))]
    QuadCountIsTooLarge { backtrace: Backtrace },

    #[snafu(display("Texture is empty: {}x{}", size.x, size.y))]
    TextureIsEmpty { size: Vec2, backtrace: Backtrace },
    #[snafu(display(
        "Mesh with quad count of {} will take {} megabytes of video memory for vertices alone. \
        Generally, to render large meshes you want to subdivide the data into smaller, separate \
        meshes and render each of those individually",
        quad_limit,
        desired_mbytes
    ))]
    VertexBufferIsTooLarge {
        quad_limit: u32,
        desired_mbytes: f32,
        backtrace: Backtrace,
    },
}

/// This is a wrapper for a vertex and index buffers used to build a static sprites mesh quad by quad.
///
/// Example for a simple tile map:
/// ```
/// use stabilkon::*;
/// use tetra::{
///     graphics::{
///         mesh::{IndexBuffer, Mesh, Vertex, VertexBuffer},
///         Color, Rectangle, Texture,
///     },
///     math::Vec2,
///     Context, TetraError,
/// };
/// # fn main() -> tetra::Result<()> {
/// # let mut ctx = tetra::ContextBuilder::new("", 1, 1).build()?;
/// // Load texture atlas with tile images:
/// let tiles_texture_atlas = Texture::new(&mut ctx, "./tests/resources/forest_tiles.png")?;
/// // Single tile is 32×32:
/// let tile_size = 32.0_f32;
/// // Let's make test map 256×256;
/// let map_size = Vec2::from(256);
/// // Calculate required quad limit for the map:
/// let quad_count = map_size.x * map_size.y;
/// // Pick grass tile from atlas, which is lockated at the very top-left of texture atlas.
/// let grass_tile_source = Rectangle::new(0.0, 0.0, 32.0, 32.0);
/// // Let's draw imaginary grass tile which is located at the very top-left of texture atlas.
/// let grass_tile_source = Rectangle::new(0.0, 0.0, 32.0, 32.0);
/// // When adding a quad to a mesh builder, you can control UV flipping with `UvFlip` parameter.
/// // By default the usual left-to-right, bottom-to-top system is used.
/// // But we decided to use left-to-right, top-to-bottom coordinate system in Rectangle creation above, so when
/// // adding quads using `grass_tile_source` a value of `UvFlip::Vertical` should be supplied.  
///
/// // Create a mesh builder for an indexed mesh capable of holding entire map...
/// let mut quad_index = 0_u32;
/// let mut mesh_builder = MeshBuilder::new(tiles_texture_atlas, quad_count)?;
/// // ... and fill it with grass tile:
/// for y in 0..map_size.y {
///     for x in 0..map_size.x {
///         let position = Vec2::new(x as f32, y as f32) * tile_size;
///         mesh_builder.set_pos_color_source(quad_index, position, Color::WHITE, grass_tile_source, UvFlip::Vertical);
///         quad_index += 1;
///     }
/// }
/// // Finally, create a mesh consisting of quads covered with grass tile texture region:
/// let (mesh, mesh_vb) = mesh_builder.create_mesh(&mut ctx)?;
/// // All done, now you can use this mesh as usual!
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct MeshBuilder {
    texture_size: Vec2,
    indices: Option<Vec<u32>>,
    vertices: Vec<Vertex>,
    quad_limit: u32,
    use_indices: bool,
    vertices_per_quad: u32,
    max_vertices: u32,
}

#[cfg(feature = "tetra")]
impl MeshBuilder {
    /// Creates mesh from all the added quads.
    ///
    /// Returns mesh's new vertex buffer, so you can call `set_data` if an update is needed later.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the underlying graphics API encounters an error when allocating vertex or index buffer.
    pub fn create_mesh(
        &self,
        ctx: &mut tetra::Context,
        texture: tetra::graphics::Texture,
    ) -> tetra::Result<(
        tetra::graphics::mesh::Mesh,
        tetra::graphics::mesh::VertexBuffer,
    )> {
        use tetra::graphics::mesh::*;
        let vertex_buffer = VertexBuffer::new(ctx, &self.vertices)?;
        let mut mesh = if let Some(index_buffer) = &self.indices {
            Mesh::indexed(vertex_buffer.clone(), IndexBuffer::new(ctx, index_buffer)?)
        } else {
            Mesh::new(vertex_buffer.clone())
        };
        mesh.set_texture(texture);
        Ok((mesh, vertex_buffer))
    }

    /// Changes the specified mesh to use texture, vertex and index buffers of this builder.
    /// Don't forget to set mesh's texture if needed.
    ///
    /// Returns mesh's new vertex buffer, so you can call `set_data` if an update is needed later.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the underlying graphics API encounters an error when allocating vertex or index buffer.
    pub fn update_mesh(
        &self,
        ctx: &mut tetra::Context,
        mesh: &mut tetra::graphics::mesh::Mesh,
    ) -> tetra::Result<tetra::graphics::mesh::VertexBuffer> {
        use tetra::graphics::mesh::*;
        let vertex_buffer = VertexBuffer::new(ctx, &self.vertices)?;
        if let Some(index_buffer) = &self.indices {
            mesh.set_index_buffer(IndexBuffer::new(ctx, index_buffer)?);
        } else {
            mesh.reset_index_buffer();
        }
        mesh.set_vertex_buffer(vertex_buffer.clone());
        Ok(vertex_buffer)
    }
}

impl MeshBuilder {
    /// Creates a mesh builder for an indexed mesh capable of holding exactly `quad_limit` quads.
    ///
    /// Note that indices and vertices are allocated immediately for the entire `quad_limit`
    /// regardless of actual `push` call count.
    ///
    /// * `texture` - This is a texture atlas referenced by quads in their `source` parameter.
    /// * `quad_limit` - Amount of quads in the built static mesh. For safest allocations,
    /// try not to go over 32 MB of needed VRAM for a single mesh, which should be 1 048 576 quads.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture_size` is < 1 or `quad_limit` is too high.
    #[inline]
    pub fn new(texture_size: Vec2, quad_limit: u32) -> Result<Self> {
        Self::create(texture_size, quad_limit, true)
    }

    /// Creates a mesh builder for a mesh without indices capable of holding exactly `quad_limit` quads.
    ///
    /// Note that vertices are allocated immediately for the entire `quad_limit`
    /// regardless of actual `push` call count.
    ///
    /// * `texture` - This is a texture atlas referenced by quads in their `source` parameter.
    /// * `quad_limit` - Amount of quads in the built static mesh. For safest allocations,
    /// try not to go over 32 MB of needed VRAM for a single mesh, which should be 1 048 576 quads.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture_size` is < 1 or `quad_limit` is too high.
    #[inline]
    pub fn new_without_indices(texture_size: Vec2, quad_limit: u32) -> Result<Self> {
        Self::create(texture_size, quad_limit, false)
    }

    /// Creates a mesh builder from the existing vertices and indices.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture` is empty.
    pub fn from_texture_vertices_indices(
        texture_size: Vec2,
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
    ) -> Result<Self> {
        ensure!(
            texture_size.x >= 1.0 && texture_size.y >= 1.0,
            TextureIsEmpty { size: texture_size }
        );
        let use_indices = indices.is_some();
        let vertices_per_quad = vertices_per_quad(use_indices);
        let max_vertices = vertices.len() as u32;
        let quad_limit = max_vertices / vertices_per_quad;
        Ok(Self {
            texture_size,
            indices,
            vertices,
            quad_limit,
            use_indices,
            vertices_per_quad,
            max_vertices,
        })
    }

    pub(crate) fn create(texture_size: Vec2, quad_limit: u32, use_indices: bool) -> Result<Self> {
        ensure!(
            texture_size.x >= 1.0 && texture_size.y >= 1.0,
            TextureIsEmpty { size: texture_size }
        );
        // Sanity check for quad_limit:
        let desired_mbytes: f32 = ((f64::from(quad_limit) * std::mem::size_of::<Vertex>() as f64)
            / (1024.0 * 1024.0)) as f32;
        ensure!(
            desired_mbytes <= MAX_VERTEX_BUFFER_SIZE_MBYTES,
            VertexBufferIsTooLarge {
                quad_limit,
                desired_mbytes
            }
        );

        let indices = if use_indices {
            Some(generate_quad_indices(quad_limit)?)
        } else {
            None
        };
        let vertices_per_quad = vertices_per_quad(use_indices);
        let max_vertices = total_vertices_in_quads(quad_limit, use_indices)?;
        let vertices: Vec<Vertex> = vec![VERTEX_ZERO; max_vertices as usize];
        Ok(Self {
            texture_size,
            indices,
            vertices,
            quad_limit,
            use_indices,
            vertices_per_quad,
            max_vertices,
        })
    }

    /// Gets the reference to the indices which will be stored in an index buffer after a `create_mesh` call.
    ///
    /// Index vec is pre-allocated and will contain valid indices for the entire `quad_limit` of quads.
    #[inline]
    #[must_use]
    pub const fn indices(&self) -> Option<&Vec<u32>> {
        self.indices.as_ref()
    }

    /// Gets the total amount of quads in the vertex buffer.
    #[inline]
    #[must_use]
    pub const fn quad_limit(&self) -> u32 {
        self.quad_limit
    }

    /// Gets the reference to the vertices which will be stored in a vertex buffer after a `create_mesh` call.
    ///
    /// Vertex vec is pre-allocated for the entire `quad_limit` of quads,
    /// with currently unused vertices set to `Vertex::default`.
    #[inline]
    #[must_use]
    pub const fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    /// Gets the total amount of vertices in the vertex buffer.
    #[inline]
    #[must_use]
    pub const fn vertices_limit(&self) -> u32 {
        self.max_vertices
    }

    /// Gets the amount of vertices used per single quad: 4 if this builder uses indices, 6 otherwise.
    #[inline]
    #[must_use]
    pub const fn vertices_per_quad(&self) -> u32 {
        self.vertices_per_quad
    }

    #[inline]
    /// Sets all added quad vertices to a default vertex data.
    pub fn clear(&mut self) {
        for item in &mut self.vertices {
            *item = VERTEX_ZERO;
        }
    }

    /// Consumes this builder and returns its vertices and indices.
    #[inline]
    #[must_use]
    pub fn extract_vertices_and_indices(self) -> (Vec<Vertex>, Option<Vec<u32>>) {
        (self.vertices, self.indices)
    }

    /// Changes quad at the given index to use the specified draw params.
    /// Returns true if the given quad index was in vertices range and vertices were set correctly; false otherwise.
    pub fn set<T: QuadDrawParams>(&mut self, quad_index: u32, draw_params: &T) -> bool {
        let vertices_per_quad = self.vertices_per_quad();
        let target_offset = quad_index * vertices_per_quad;
        if target_offset + vertices_per_quad <= self.max_vertices {
            draw_params.set_vertices(
                self.texture_size,
                self.use_indices,
                target_offset as usize,
                &mut self.vertices,
            );
            true
        } else {
            false
        }
    }

    /// Changes quad at the given index to use the specified position, color and texture source rectangle.
    /// Returns true if the given quad index was in vertices range and vertices were set correctly; false otherwise.
    ///
    /// * `quad_index` - Infex of the quad to set. Quads start at 0 and end at `limit` - 1.
    /// * `position` - Quad position, top-left corner.
    /// * `color` - Quad vertices color.
    /// * `source` - Texture source rectangle. Along with `flip`, determines which part of the texture will drawn.
    /// * `flip` - UV flip mode.
    #[inline]
    pub fn set_pos_color_source(
        &mut self,
        quad_index: u32,
        position: Vec2,
        color: Color,
        source: Rectangle,
        flip: UvFlip,
    ) -> bool {
        let draw_info = PosColorSource {
            position,
            color,
            source,
            flip,
        };
        self.set(quad_index, &draw_info)
    }

    /// Changes quad at the given index to use the specified position, color, size and texture source rectangle.
    /// Returns true if the given quad index was in vertices range and vertices were set correctly; false otherwise.
    ///
    /// * `quad_index` - Infex of the quad to set. Quads start at 0 and end at `limit` - 1.
    /// * `position` - Quad position, top-left corner.
    /// * `color` - Quad vertices color.
    /// * `size` - Destination size, used for absolute scaling.
    /// * `source` - Texture source rectangle. Along with `flip`, determines which part of the texture will drawn.
    /// * `flip` - UV flip mode.
    #[inline]
    pub fn set_pos_color_size_source(
        &mut self,
        quad_index: u32,
        position: Vec2,
        color: Color,
        size: Vec2,
        source: Rectangle,
        flip: UvFlip,
    ) -> bool {
        let draw_info = PosColorSizeSource {
            position,
            color,
            size,
            source,
            flip,
        };
        self.set(quad_index, &draw_info)
    }
}

/// Generates indices for the given amount of quads.
///
/// #Errors
///
/// Will return `Err` if `quad_count` multiplied by 6 overflows u32.
pub fn generate_quad_indices(quad_count: u32) -> Result<Vec<u32>> {
    let length = match quad_count.checked_mul(6) {
        Some(total_indices) => Ok(total_indices),
        None => QuadCountIsTooLarge {}.fail(),
    }?;
    let mut indices = vec![0_u32; length as usize];
    let mut offset: usize = 0;
    let mut index_value: u32 = 0;
    while offset < length as usize {
        indices[offset] = index_value;
        indices[offset + 1] = index_value + 1;
        indices[offset + 2] = index_value + 2;
        indices[offset + 3] = index_value + 2;
        indices[offset + 4] = index_value + 3;
        indices[offset + 5] = index_value;
        index_value += 4;
        offset += 6;
    }
    Ok(indices)
}

/// Gets the amount of vertices used per single quad: 4 when using indices, 6 otherwise.
#[inline]
pub const fn vertices_per_quad(use_indices: bool) -> u32 {
    if use_indices {
        4
    } else {
        6
    }
}

/// Gets the amount of vertices needed to draw given quad count.
///
/// #Errors
///
/// Will return `Err` if `quad_count` multiplied by vertices per quad overflows u32.
#[inline]
pub fn total_vertices_in_quads(quad_count: u32, use_indices: bool) -> Result<u32> {
    match quad_count.checked_mul(vertices_per_quad(use_indices)) {
        Some(total_vertices) => Ok(total_vertices),
        None => QuadCountIsTooLarge {}.fail(),
    }
}