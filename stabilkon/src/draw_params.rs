use crate::vertices_per_quad;

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

static VEC2_ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

/// Determines how UVs flip and the resulting texture coordinate system.
///
/// Can be used to change how `source` parameter is treated when adding quads to a builder.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UvFlip {
    /// Results in usual left-to-right, bottom-to-top (↑→).
    None,
    /// Results in right-to-left, bottom-to-top (←↑).
    Horizontal,
    /// Results in left-to-right, top-to-bottom (↓→).
    Vertical,
    /// Results in right-to-left, top-to-bottom (←↓).
    Both,
}

/// Used to represent a single quad for a static sprites mesh.
pub trait QuadDrawParams {
    /// Gets vertices color.
    fn get_color(&self) -> Color;

    /// Calculates corner points for transformations contained in this `QuadDrawParams`,
    /// clockwise from position.
    fn corner_points(
        &self,
        texture_size: Vec2,
        c1: &mut Vec2,
        c2: &mut Vec2,
        c3: &mut Vec2,
        c4: &mut Vec2,
    );

    /// Calculates top-left and bottom-right UVs using texture source information.
    fn uvs(&self, texture_size: Vec2, top_left: &mut Vec2, bottom_right: &mut Vec2);

    /// Calculates vertices and sets them in the given vertex buffer starting at the specified offset.
    ///
    /// * `texture_size` - Texture dimensions.
    /// * `use_indices` - If set to true, quad will consist of 4 vertices; otherwise, 6 vertices will be used.
    /// * `vertex_offset` - Index at which quad vertices will be set in `vertices` buffer.
    /// * `vertices` - Vertices buffer, must be pre-allocated.
    fn set_vertices(
        &self,
        texture_size: Vec2,
        use_indices: bool,
        vertex_offset: usize,
        vertices: &mut Vec<Vertex>,
    ) {
        let mut c1_position = VEC2_ZERO;
        let mut c2_position = VEC2_ZERO;
        let mut c3_position = VEC2_ZERO;
        let mut c4_position = VEC2_ZERO;
        self.corner_points(
            texture_size,
            &mut c1_position,
            &mut c2_position,
            &mut c3_position,
            &mut c4_position,
        );
        let mut c1_uv = VEC2_ZERO;
        let mut c2_uv = VEC2_ZERO;
        let mut c3_uv = VEC2_ZERO;
        let mut c4_uv = VEC2_ZERO;
        self.uvs(texture_size, &mut c1_uv, &mut c3_uv);
        c2_uv.x = c1_uv.x;
        c2_uv.y = c3_uv.y;
        c4_uv.x = c3_uv.x;
        c4_uv.y = c1_uv.y;

        let (c1, c2, c3, c4) = make_vertices(
            self.get_color(),
            c1_position,
            c2_position,
            c3_position,
            c4_position,
            c1_uv,
            c2_uv,
            c3_uv,
            c4_uv,
        );
        vertices[vertex_offset] = c1;
        vertices[vertex_offset + 1] = c2;
        vertices[vertex_offset + 2] = c3;
        if use_indices {
            vertices[vertex_offset + 3] = c4;
        } else {
            vertices[vertex_offset + 3] = c3;
            vertices[vertex_offset + 4] = c4;
            vertices[vertex_offset + 5] = c1;
        }
    }

    /// Calculates and returns ordered vertices.
    ///
    /// * `texture_size` - Texture dimensions.
    /// * `use_indices` - If set to true, quad will consist of 4 vertices; otherwise, 6 vertices will be used.
    fn to_vertices(&self, texture_size: Vec2, use_indices: bool) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(vertices_per_quad(use_indices) as usize);
        self.set_vertices(texture_size, use_indices, 0, &mut vertices);
        vertices
    }
}

/// Standard quad draw info.
#[derive(Clone, Debug, PartialEq)]
pub struct PosColorSource {
    /// Quad position, top-left corner.
    pub position: Vec2,
    /// Quad vertices color.
    pub color: Color,
    /// Texture source rectangle. Along with `flip`, determines which part of the texture will drawn.
    pub source: Rectangle,
    /// UV flip mode.
    pub flip: UvFlip,
}

impl PosColorSource {
    #[inline]
    #[must_use]
    pub const fn new(position: Vec2, color: Color, source: Rectangle) -> Self {
        Self {
            position,
            color,
            source,
            flip: UvFlip::None,
        }
    }
}

impl QuadDrawParams for PosColorSource {
    fn get_color(&self) -> Color {
        self.color
    }

    fn corner_points(
        &self,
        texture_size: Vec2,
        c1: &mut Vec2,
        c2: &mut Vec2,
        c3: &mut Vec2,
        c4: &mut Vec2,
    ) {
        #[cfg(feature = "ggez")]
        let source_width = self.source.w;
        #[cfg(any(feature = "tetra", not(feature = "ggez")))]
        let source_width = self.source.width;
        let source_or_texture_width = if source_width > 0.0 {
            source_width
        } else {
            texture_size.x
        };

        #[cfg(feature = "ggez")]
        let source_height = self.source.h;
        #[cfg(any(feature = "tetra", not(feature = "ggez")))]
        let source_height = self.source.height;
        let source_or_texture_height = if source_height > 0.0 {
            source_height
        } else {
            texture_size.y
        };

        let f2 = Vec2 {
            x: self.position.x + source_or_texture_width,
            y: self.position.y + source_or_texture_height,
        };
        c1.x = self.position.x;
        c1.y = self.position.y;

        c2.x = self.position.x;
        c2.y = f2.y;

        c3.x = f2.x;
        c3.y = f2.y;

        c4.x = f2.x;
        c4.y = self.position.y;
    }

    #[inline]
    fn uvs(&self, texture_size: Vec2, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(texture_size, &self.source, self.flip, uv, uv2);
    }
}

/// Standard quad draw info with additional absolute scaling.
#[derive(Clone, Debug, PartialEq)]
pub struct PosColorSizeSource {
    /// Quad position, top-left corner.
    pub position: Vec2,
    /// Quad vertices color.
    pub color: Color,
    /// Destination size, used for absolute scaling.
    pub size: Vec2,
    /// Texture source rectangle. Along with `flip`, determines which part of the texture will drawn.
    pub source: Rectangle,
    /// UV flip mode.
    pub flip: UvFlip,
}

impl PosColorSizeSource {
    #[inline]
    #[must_use]
    pub const fn new(position: Vec2, color: Color, size: Vec2, source: Rectangle) -> Self {
        Self {
            position,
            color,
            size,
            source,
            flip: UvFlip::None,
        }
    }
}

impl QuadDrawParams for PosColorSizeSource {
    #[inline]
    fn get_color(&self) -> Color {
        self.color
    }

    fn corner_points(
        &self,
        _texture_size: Vec2,
        c1: &mut Vec2,
        c2: &mut Vec2,
        c3: &mut Vec2,
        c4: &mut Vec2,
    ) {
        let f2 = Vec2 {
            x: self.position.x + self.size.x,
            y: self.position.y + self.size.y,
        };
        c1.x = self.position.x;
        c1.y = self.position.y;

        c2.x = self.position.x;
        c2.y = f2.y;

        c3.x = f2.x;
        c3.y = f2.y;

        c4.x = f2.x;
        c4.y = self.position.y;
    }

    #[inline]
    fn uvs(&self, texture_size: Vec2, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(texture_size, &self.source, self.flip, uv, uv2);
    }
}

/// Quad info where you control everything.
#[derive(Clone, Debug, PartialEq)]
pub struct DetailedParams {
    /// Quad position, top-left corner.
    pub position: Vec2,
    /// Quad vertices color.
    pub color: Color,
    /// Offsets position and serves as a rotation center.
    pub origin: Vec2,
    /// Destination size, used for absolute scaling.
    pub size: Vec2,
    /// Scale, used for relative scaling.
    pub scale: Vec2,
    /// Rotation angle in radians.
    pub rotation: f32,
    /// Texture source rectangle. Along with `flip`, determines which part of the texture will drawn.
    pub source: Rectangle,
    /// UV flip mode.
    pub flip: UvFlip,
}

impl DetailedParams {
    #[inline]
    #[must_use]
    pub const fn new(
        position: Vec2,
        color: Color,
        origin: Vec2,
        size: Vec2,
        scale: Vec2,
        rotation: f32,
        source: Rectangle,
    ) -> Self {
        Self {
            position,
            color,
            origin,
            size,
            scale,
            rotation,
            source,
            flip: UvFlip::None,
        }
    }
}

impl QuadDrawParams for DetailedParams {
    fn corner_points(
        &self,
        _texture_size: Vec2,
        c1: &mut Vec2,
        c2: &mut Vec2,
        c3: &mut Vec2,
        c4: &mut Vec2,
    ) {
        // bottom left and top right corner points relative to origin
        let world_origin = Vec2 {
            x: self.position.x + self.origin.x,
            y: self.position.y + self.origin.y,
        };

        let mut f = Vec2 {
            x: -self.origin.x,
            y: -self.origin.y,
        };
        let mut f2 = Vec2 {
            x: self.size.x - self.origin.x,
            y: self.size.y - self.origin.y,
        };
        if (self.scale.x - 1.0).abs() > 0.001 || (self.scale.y - 1.0).abs() > 0.001 {
            f.x *= self.scale.x;
            f.y *= self.scale.y;
            f2.x *= self.scale.x;
            f2.y *= self.scale.y;
        }

        // construct corner points, start from top left and go counter clockwise
        let p1 = f;
        let p2 = Vec2 { x: f.x, y: f2.y };
        let p3 = Vec2 { x: f2.x, y: f2.y };
        let p4 = Vec2 { x: f2.x, y: f.y };

        if self.rotation == 0.0 {
            c1.x = p1.x;
            c1.y = p1.y;

            c2.x = p2.x;
            c2.y = p2.y;

            c3.x = p3.x;
            c3.y = p3.y;

            c4.x = p4.x;
            c4.y = p4.y;
        } else {
            let cos = self.rotation.cos();
            let sin = self.rotation.sin();

            c1.x = cos * p1.x - sin * p1.y;
            c1.y = sin.mul_add(p1.x, cos * p1.y);

            c2.x = cos * p2.x - sin * p2.y;
            c2.y = sin.mul_add(p2.x, cos * p2.y);

            c3.x = cos * p3.x - sin * p3.y;
            c3.y = sin.mul_add(p3.x, cos * p3.y);

            c4.x = c1.x + (c3.x - c2.x);
            c4.y = c3.y - (c2.y - c1.y);
        }

        c1.x += world_origin.x;
        c1.y += world_origin.y;
        c2.x += world_origin.x;
        c2.y += world_origin.y;
        c3.x += world_origin.x;
        c3.y += world_origin.y;
        c4.x += world_origin.x;
        c4.y += world_origin.y;
    }

    #[inline]
    fn uvs(&self, texture_size: Vec2, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(texture_size, &self.source, self.flip, uv, uv2);
    }

    #[inline]
    fn get_color(&self) -> Color {
        self.color
    }
}

pub fn calculate_uvs_with_source(
    texture_size: Vec2,
    source: &Rectangle,
    flip: UvFlip,
    uv: &mut Vec2,
    uv2: &mut Vec2,
) {
    if texture_size.x > 0.0 && texture_size.y > 0.0 {
        // Tetra calculates UV like this for its left-to-right top-to-bottom texcoords:
        // let mut u = source.x / texture_size.x;
        // let mut v = source.y / texture_size.y;
        // let mut u2 = source.right() / texture_size.x;
        // let mut v2 = source.bottom() / texture_size.y;
        // Instead, we will conform to OpenGL default left-to-right bottom-to-top texcoords and
        // let end users to flip UVs how they see fit:
        let mut u = source.x / texture_size.x;
        let mut v = source.bottom() / texture_size.y;
        let mut u2 = source.right() / texture_size.x;
        let mut v2 = source.y / texture_size.y;
        flip_uvs(flip, &mut u, &mut v, &mut u2, &mut v2);
        uv.x = u;
        uv.y = v;
        uv2.x = u2;
        uv2.y = v2;
    } else {
        uv.x = 0.0;
        uv.y = 1.0;
        uv2.x = 1.0;
        uv2.y = 0.0;
    }
}

#[inline]
pub fn flip_uvs<'uvs, T>(
    flip: UvFlip,
    u: &'uvs mut T,
    v: &'uvs mut T,
    u2: &'uvs mut T,
    v2: &'uvs mut T,
) {
    if flip == UvFlip::Horizontal || flip == UvFlip::Both {
        std::mem::swap(u, u2);
    }
    if flip == UvFlip::Vertical || flip == UvFlip::Both {
        std::mem::swap(v, v2);
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "ggez")]
#[inline]
pub(crate) fn make_vertices(
    color: Color,
    c1_position: Vec2,
    c2_position: Vec2,
    c3_position: Vec2,
    c4_position: Vec2,
    c1_uv: Vec2,
    c2_uv: Vec2,
    c3_uv: Vec2,
    c4_uv: Vec2,
) -> (Vertex, Vertex, Vertex, Vertex) {
    let c1 = Vertex {
        color: color.into(),
        pos: c1_position.into(),
        uv: c1_uv.into(),
    };
    let c2 = Vertex {
        color: color.into(),
        pos: c2_position.into(),
        uv: c2_uv.into(),
    };
    let c3 = Vertex {
        color: color.into(),
        pos: c3_position.into(),
        uv: c3_uv.into(),
    };
    let c4 = Vertex {
        color: color.into(),
        pos: c4_position.into(),
        uv: c4_uv.into(),
    };
    (c1, c2, c3, c4)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "tetra")]
#[inline]
pub(crate) fn make_vertices(
    color: Color,
    c1_position: Vec2,
    c2_position: Vec2,
    c3_position: Vec2,
    c4_position: Vec2,
    c1_uv: Vec2,
    c2_uv: Vec2,
    c3_uv: Vec2,
    c4_uv: Vec2,
) -> (Vertex, Vertex, Vertex, Vertex) {
    let c1 = Vertex {
        color,
        position: c1_position,
        uv: c1_uv,
    };
    let c2 = Vertex {
        color,
        position: c2_position,
        uv: c2_uv,
    };
    let c3 = Vertex {
        color,
        position: c3_position,
        uv: c3_uv,
    };
    let c4 = Vertex {
        color,
        position: c4_position,
        uv: c4_uv,
    };
    (c1, c2, c3, c4)
}

#[cfg(not(any(feature = "ggez", feature = "tetra")))]
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn make_vertices(
    color: Color,
    c1_position: Vec2,
    c2_position: Vec2,
    c3_position: Vec2,
    c4_position: Vec2,
    c1_uv: Vec2,
    c2_uv: Vec2,
    c3_uv: Vec2,
    c4_uv: Vec2,
) -> (Vertex, Vertex, Vertex, Vertex) {
    let c1 = Vertex {
        color,
        position: c1_position,
        uv: c1_uv,
    };
    let c2 = Vertex {
        color,
        position: c2_position,
        uv: c2_uv,
    };
    let c3 = Vertex {
        color,
        position: c3_position,
        uv: c3_uv,
    };
    let c4 = Vertex {
        color,
        position: c4_position,
        uv: c4_uv,
    };
    (c1, c2, c3, c4)
}
