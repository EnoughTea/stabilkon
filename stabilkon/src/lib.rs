#![crate_type = "lib"]

mod common_types;
mod draw_params;

pub use common_types::*;
pub use draw_params::*;
pub use mint;

use snafu::{ensure, Backtrace, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum Error {
    #[snafu(display("Quad count is too large"))]
    QuadCountIsTooLarge { backtrace: Backtrace },

    #[snafu(display("Texture size is invalid: {}x{}", size.x, size.y))]
    InvalidTextureSize { size: Vec2, backtrace: Backtrace },

    #[snafu(display(
        "Vertex buffer with length '{}' is too large. \
        Generally, to render large meshes you want to subdivide the data into smaller, separate \
        meshes and render each of those individually",
        length
    ))]
    VertexBufferIsTooLarge { length: usize, backtrace: Backtrace },
}

/// This is a wrapper for a vertex and index buffers used to build a static mesh quad by quad.
///
/// It is expected to be used with a custom vertex type with implemented `From<PosUvColor>`,
/// with support for ggez and Tetra vertex types provided via crate features.
///
/// Just make sure that given vertex type only contains values and does not contain references,
/// or constructor will fail spectacularly: internally, vertex buffer is inited with zeroed memory
/// by `MaybeUninit::zeroed()`, due to ggez not having `Default` trait on its vertex type.
///
/// # Example
///
/// A simple tile map with internal vertex type called `PosUvColor`:
///
/// ```
/// use stabilkon::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load texture atlas with tile images:
/// let tiles_texture_atlas_size = [288.0, 128.0];
/// // We won't be using custom shaders and such, - let the mesh builder fix UVs for us:
/// let use_half_pixel_offset = true;
///
/// // Single tile is 32×32:
/// let tile_size = 32.0_f32;
/// // Let's make test map 256×256;
/// let map_size = [256, 256];
/// // Calculate required quad limit for the map:
/// let quad_count = map_size[0] * map_size[1];
/// // Standard white color, means tile images will be drawn as-is.
/// let white_color = [1.0_f32, 1.0, 1.0, 1.0];
///
/// // Pick grass tile from atlas, which is lockated at the very top-left of texture atlas.
/// let grass_tile_source = [0.0, 0.0, 32.0, 32.0];
/// // Let's draw imaginary grass tile which is located at the very top-left of texture atlas.
/// let grass_tile_source = [0.0, 0.0, 32.0, 32.0];
/// // When adding a quad to a mesh builder, you can control UV flipping with `UvFlip` parameter.
/// // By default the usual left-to-right, bottom-to-top system is used.
/// // But we decided to use left-to-right, top-to-bottom coordinate system in Rectangle creation above, so when
/// // adding quads using `grass_tile_source` a value of `UvFlip::Vertical` should be supplied.  
///
/// // Create a mesh builder for an indexed mesh capable of holding entire map...
/// let mut quad_index = 0_u32;
/// let mut mesh_builder: MeshFromQuads<PosUvColor> =
///     MeshFromQuads::new(tiles_texture_atlas_size, use_half_pixel_offset, quad_count)?;
/// // ... and fill it with grass tile:
/// for y in 0..map_size[1] {
///     for x in 0..map_size[0] {
///         let position = [x as f32 * tile_size, y as f32 * tile_size];
///         mesh_builder.set_pos_color_source(quad_index, position, white_color, grass_tile_source, UvFlip::Vertical);
///         quad_index += 1;
///     }
/// }
/// // Finally, create a mesh consisting of quads covered with grass tile texture region:
/// let (vertices, indices) = mesh_builder.into_vertices_and_indices();
/// // All done, now you can draw these vertices using any API you want.
/// // Both vertices and indices are in clockwise order.
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct MeshFromQuads<TVertex>
where
    TVertex: From<PosUvColor>,
{
    texture_size: Vec2,
    use_half_pixel_offset: bool,
    indices: Option<Vec<u32>>,
    vertices: Vec<TVertex>,
    quad_limit: u32,
    use_indices: bool,
    vertices_per_quad: u32,
    max_vertices: u32,
}

#[cfg(feature = "ggez")]
impl MeshFromQuads<ggez::graphics::Vertex> {
    /// Creates a ggez mesh from all the added quads.
    ///
    /// # Errors
    ///
    /// Will return `Err` if builder has no indices.
    pub fn create_mesh(
        &self,
        ctx: &mut ggez::Context,
        texture: ggez::graphics::Image,
    ) -> ggez::GameResult<ggez::graphics::Mesh> {
        use ggez::graphics::Mesh;
        match self.indices.as_ref() {
            Some(indices) => Mesh::from_raw(ctx, &self.vertices, indices, Some(texture)),
            None => Err(ggez::GameError::CustomError(
                "Unindexed meshes are not supported".to_owned(),
            )),
        }
    }

    /// Changes the specified ggez mesh to use vertex and index buffers of this builder.
    /// Don't forget to set mesh's texture if needed.
    ///
    /// # Errors
    ///
    /// Will return `Err` if builder has no indices.
    pub fn update_mesh(
        &self,
        ctx: &mut ggez::Context,
        mesh: &mut ggez::graphics::Mesh,
    ) -> ggez::GameResult<()> {
        match self.indices.as_ref() {
            Some(indices) => {
                mesh.set_vertices(ctx, &self.vertices, indices);
                Ok(())
            }
            None => Err(ggez::GameError::CustomError(
                "Unindexed meshes are not supported".to_owned(),
            )),
        }
    }
}

#[cfg(feature = "tetra")]
impl MeshFromQuads<tetra::graphics::mesh::Vertex> {
    /// Creates a Tetra mesh from all the added quads.
    ///
    /// Returns both the mesh and its new vertex buffer. You can use its `set_data` if an update is needed later.
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
        use tetra::graphics::mesh::{IndexBuffer, Mesh, VertexBuffer};
        let vertex_buffer = VertexBuffer::new(ctx, &self.vertices)?;
        let mut mesh = if let Some(index_buffer) = &self.indices {
            Mesh::indexed(vertex_buffer.clone(), IndexBuffer::new(ctx, index_buffer)?)
        } else {
            Mesh::new(vertex_buffer.clone())
        };
        mesh.set_texture(texture);
        Ok((mesh, vertex_buffer))
    }

    /// Changes the specified Tetra mesh to use texture, vertex and index buffers of this builder.
    /// Don't forget to set mesh's texture if needed.
    ///
    /// Returns mesh's new vertex buffer. You can use its `set_data` if an update is needed later.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the underlying graphics API encounters an error when allocating vertex or index buffer.
    pub fn update_mesh(
        &self,
        ctx: &mut tetra::Context,
        mesh: &mut tetra::graphics::mesh::Mesh,
    ) -> tetra::Result<tetra::graphics::mesh::VertexBuffer> {
        use tetra::graphics::mesh::{IndexBuffer, VertexBuffer};
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

impl<TVertex> MeshFromQuads<TVertex>
where
    TVertex: Clone + From<PosUvColor>,
{
    /// Creates a mesh builder for an indexed mesh capable of holding exactly `quad_limit` quads.
    ///
    /// Note that indices and vertices are allocated immediately for the entire `quad_limit`
    /// regardless of actual `push` call count.
    ///
    /// * `texture_size` - Size of the texture atlas which will be used by the resulting mesh.
    /// * `use_half_pixel_offset` - If set to true, applies [half pixel correction]
    /// (https://docs.microsoft.com/en-us/windows/win32/direct3d9/directly-mapping-texels-to-pixels) directly to UVs.
    /// It is better to use padded texture atlas with this fix,
    /// otherwise only half of border pixels will be displayed. This is often imperceptible, unlike bleeding,
    /// but keep it in mind.
    /// If set to false, expects end users to deal with texture bleeding themselves,
    /// e.g. with correct texture sampling or shifting viewport by half a pixel.
    /// * `quad_limit` - Amount of quads in the built static mesh. For safest allocations,
    /// try not to go over 32 MB of needed VRAM for a single mesh.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture_size` is < 1 or `quad_limit` is too high.
    #[inline]
    pub fn new<T: Into<Vec2>>(
        texture_size: T,
        use_half_pixel_offset: bool,
        quad_limit: u32,
    ) -> Result<Self> {
        Self::create(texture_size, use_half_pixel_offset, quad_limit, true)
    }

    /// Creates a mesh builder for a mesh without indices capable of holding exactly `quad_limit` quads.
    ///
    /// Note that vertices are allocated immediately for the entire `quad_limit`
    /// regardless of actual `push` call count.
    ///
    /// * `texture_size` - Size of the texture atlas which will be used by the resulting mesh.
    /// * `use_half_pixel_offset` - If set to true, applies [half pixel correction]
    /// (https://docs.microsoft.com/en-us/windows/win32/direct3d9/directly-mapping-texels-to-pixels) directly to UVs.
    /// It is better to use padded texture atlas with this fix,
    /// otherwise only half of border pixels will be displayed. This is often imperceptible, unlike bleeding,
    /// but keep it in mind.
    /// If set to false, expects end users to deal with texture bleeding themselves,
    /// e.g. with correct texture sampling or shifting viewport by half a pixel.
    /// * `quad_limit` - Amount of quads in the built static mesh. For safest allocations,
    /// try not to go over 32 MB of needed VRAM for a single mesh.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture_size` is < 1 or `quad_limit` is too high.
    #[inline]
    pub fn new_without_indices<T: Into<Vec2>>(
        texture_size: T,
        use_half_pixel_offset: bool,
        quad_limit: u32,
    ) -> Result<Self> {
        Self::create(texture_size, use_half_pixel_offset, quad_limit, false)
    }

    /// Creates a mesh builder from the existing vertices and indices.
    ///
    /// * `texture_size` - Size of the texture atlas which will be used by the resulting mesh.
    /// * `use_half_pixel_offset` - If set to true, applies [half pixel correction]
    /// (https://docs.microsoft.com/en-us/windows/win32/direct3d9/directly-mapping-texels-to-pixels) directly to UVs.
    /// It is better to use padded texture atlas with this fix,
    /// otherwise only half of border pixels will be displayed. This is often imperceptible, unlike bleeding,
    /// but keep it in mind.
    /// If set to false, expects end users to deal with texture bleeding themselves,
    /// e.g. with correct texture sampling or shifting viewport by half a pixel.
    /// * `vertices` - Existing vertices to modify.
    /// * `indices` - Indices for the given existing vertices.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `texture_size` is < 1.
    pub fn from_texture_vertices_indices<T: Into<Vec2>>(
        texture_size: T,
        use_half_pixel_offset: bool,
        vertices: Vec<TVertex>,
        indices: Option<Vec<u32>>,
    ) -> Result<Self> {
        let texture_size_vec: Vec2 = texture_size.into();
        ensure!(
            texture_size_vec.x >= 1.0 && texture_size_vec.y >= 1.0,
            InvalidTextureSize {
                size: texture_size_vec
            }
        );
        ensure!(
            u32::try_from(vertices.len()).is_ok(),
            VertexBufferIsTooLarge {
                length: vertices.len()
            }
        );

        let use_indices = indices.is_some();
        let vertices_per_quad = vertices_per_quad(use_indices);
        let max_vertices = vertices.len() as u32;
        let quad_limit = max_vertices / vertices_per_quad;
        Ok(Self {
            texture_size: texture_size_vec,
            use_half_pixel_offset,
            indices,
            vertices,
            quad_limit,
            use_indices,
            vertices_per_quad,
            max_vertices,
        })
    }

    pub(crate) fn create<T: Into<Vec2>>(
        texture_size: T,
        use_half_pixel_offset: bool,
        quad_limit: u32,
        use_indices: bool,
    ) -> Result<Self> {
        let texture_size_vec: Vec2 = texture_size.into();
        ensure!(
            texture_size_vec.x >= 1.0 && texture_size_vec.y >= 1.0,
            InvalidTextureSize {
                size: texture_size_vec
            }
        );

        let indices = if use_indices {
            Some(generate_quad_indices(quad_limit)?)
        } else {
            None
        };
        let vertices_per_quad = vertices_per_quad(use_indices);
        let max_vertices = total_vertices_in_quads(quad_limit, use_indices)?;
        let zeroed_vertex = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        let vertices: Vec<TVertex> = vec![zeroed_vertex; max_vertices as usize];
        Ok(Self {
            texture_size: texture_size_vec,
            use_half_pixel_offset,
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
    /// Indices draw the vertices in clockwise order.
    /// Index vec is pre-allocated and will contain valid indices for the entire `quad_limit` of quads.
    #[inline]
    #[must_use]
    pub fn indices(&self) -> Option<&Vec<u32>> {
        self.indices.as_ref()
    }

    /// Gets the total amount of quads in the vertex buffer.
    #[inline]
    #[must_use]
    pub fn quad_limit(&self) -> u32 {
        self.quad_limit
    }

    /// Gets the reference to the vertices which will be stored in a vertex buffer after a `create_mesh` call.
    ///
    /// Vertices are in clockwise order.
    /// Vertex vec is pre-allocated for the entire `quad_limit` of quads,
    /// with currently unused vertices set to `Vertex::default`.
    #[inline]
    #[must_use]
    pub fn vertices(&self) -> &Vec<TVertex> {
        &self.vertices
    }

    /// Gets the total amount of vertices in the vertex buffer.
    #[inline]
    #[must_use]
    pub fn vertices_limit(&self) -> u32 {
        self.max_vertices
    }

    /// Gets the amount of vertices used per single quad: 4 if this builder uses indices, 6 otherwise.
    #[inline]
    #[must_use]
    pub fn vertices_per_quad(&self) -> u32 {
        self.vertices_per_quad
    }

    #[inline]
    /// Sets all added quad vertices to a default vertex data.
    pub fn clear(&mut self) {
        for item in &mut self.vertices {
            *item = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        }
    }

    /// Consumes this builder and returns its vertices and indices.
    ///
    /// Both vertices and indices are in clockwise order.
    #[inline]
    #[must_use]
    pub fn into_vertices_and_indices(self) -> (Vec<TVertex>, Option<Vec<u32>>) {
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
                self.use_half_pixel_offset,
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
    pub fn set_pos_color_source<TColor, TRect, TVec2>(
        &mut self,
        quad_index: u32,
        position: TVec2,
        color: TColor,
        source: TRect,
        flip: UvFlip,
    ) -> bool
    where
        TColor: Into<Color>,
        TRect: Into<Rectangle>,
        TVec2: Into<Vec2>,
    {
        let draw_info = PosColorSource::new(position, color, source, flip);
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
    pub fn set_pos_color_size_source<TColor, TRect, TVec2>(
        &mut self,
        quad_index: u32,
        position: TVec2,
        color: TColor,
        size: TVec2,
        source: TRect,
        flip: UvFlip,
    ) -> bool
    where
        TColor: Into<Color>,
        TRect: Into<Rectangle>,
        TVec2: Into<Vec2>,
    {
        let draw_info = PosColorSizeSource::new(position, color, size, source, flip);
        self.set(quad_index, &draw_info)
    }
}

/// Generates indices for the given amount of quads.
///
/// # Errors
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
#[must_use]
pub const fn vertices_per_quad(use_indices: bool) -> u32 {
    if use_indices {
        4
    } else {
        6
    }
}

/// Gets the amount of vertices needed to draw given quad count.
///
/// # Errors
///
/// Will return `Err` if `quad_count` multiplied by vertices per quad overflows u32.
#[inline]
pub(crate) fn total_vertices_in_quads(quad_count: u32, use_indices: bool) -> Result<u32> {
    match quad_count.checked_mul(vertices_per_quad(use_indices)) {
        Some(total_vertices) => Ok(total_vertices),
        None => QuadCountIsTooLarge {}.fail(),
    }
}
