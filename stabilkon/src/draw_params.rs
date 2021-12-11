use crate::{
    common_types::{Color, PosUvColor, Rectangle, Vec2, VEC2_ZERO},
    vertices_per_quad,
};

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

    /// Calculates corner points starting from (x, y) and going clockwise.
    fn corner_points(
        &self,
        texture_size: Vec2,
        c1: &mut Vec2,
        c2: &mut Vec2,
        c3: &mut Vec2,
        c4: &mut Vec2,
    );

    /// Calculates top-left and bottom-right UVs.
    fn uvs(
        &self,
        texture_size: Vec2,
        use_half_pixel_offset: bool,
        top_left: &mut Vec2,
        bottom_right: &mut Vec2,
    );

    /// Calculates vertices and sets them in the given vertex buffer starting at the specified offset.
    ///
    /// * `texture_size` - Size of the texture atlas which will be used by the resulting mesh.
    /// * `use_half_pixel_offset` - If set to true, applies [half pixel correction]
    /// (https://docs.microsoft.com/en-us/windows/win32/direct3d9/directly-mapping-texels-to-pixels) directly to UVs.
    /// It is better to use padded texture atlas with this fix,
    /// otherwise only half of border pixels will be displayed. This is often imperceptible, unlike bleeding,
    /// but keep it in mind.
    /// If set to false, expects end users to deal with texture bleeding themselves,
    /// e.g. with correct texture sampling or shifting viewport by half a pixel.
    /// * `use_indices` - If set to true, quad will consist of 4 vertices; otherwise, 6 vertices will be used.
    /// * `vertex_offset` - Index at which quad vertices will be set in `vertices` buffer.
    /// * `vertices` - Vertices buffer, must be pre-allocated.
    fn set_vertices<TVertex>(
        &self,
        texture_size: Vec2,
        use_half_pixel_offset: bool,
        use_indices: bool,
        vertex_offset: usize,
        vertices: &mut Vec<TVertex>,
    ) where
        TVertex: Clone + From<PosUvColor>,
    {
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
        self.uvs(texture_size, use_half_pixel_offset, &mut c1_uv, &mut c3_uv);
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

        if use_indices {
            vertices[vertex_offset] = c1;
            vertices[vertex_offset + 1] = c2;
            vertices[vertex_offset + 2] = c3;
            vertices[vertex_offset + 3] = c4;
        } else {
            vertices[vertex_offset] = c1.clone();
            vertices[vertex_offset + 1] = c2;
            vertices[vertex_offset + 2] = c3.clone();
            vertices[vertex_offset + 3] = c3;
            vertices[vertex_offset + 4] = c4;
            vertices[vertex_offset + 5] = c1;
        }
    }

    /// Calculates and returns ordered vertices.
    ///
    /// * `texture_size` - Texture dimensions.
    /// * `use_half_pixel_offset` - If set to true, applies [half pixel correction]
    /// (https://docs.microsoft.com/en-us/windows/win32/direct3d9/directly-mapping-texels-to-pixels) directly to UVs.
    /// It is better to use padded texture atlas with this fix,
    /// otherwise only half of border pixels will be displayed. This is often imperceptible, unlike bleeding,
    /// but keep it in mind.
    /// If set to false, expects end users to deal with texture bleeding themselves,
    /// e.g. with correct texture sampling or shifting viewport by half a pixel.
    /// * `use_indices` - If set to true, quad will consist of 4 vertices; otherwise, 6 vertices will be used.
    fn to_vertices<TVertex>(
        &self,
        texture_size: Vec2,
        use_half_pixel_offset: bool,
        use_indices: bool,
    ) -> Vec<TVertex>
    where
        TVertex: Clone + From<PosUvColor>,
    {
        let mut vertices = Vec::with_capacity(vertices_per_quad(use_indices) as usize);
        self.set_vertices(
            texture_size,
            use_half_pixel_offset,
            use_indices,
            0,
            &mut vertices,
        );
        vertices
    }
}

/// Represents a standard, run-of-the-mill quad.
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
    pub fn new<TColor, TRect, TVec2>(
        position: TVec2,
        color: TColor,
        source: TRect,
        flip: UvFlip,
    ) -> Self
    where
        TColor: Into<Color>,
        TRect: Into<Rectangle>,
        TVec2: Into<Vec2>,
    {
        Self {
            position: position.into(),
            color: color.into(),
            source: source.into(),
            flip,
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
        let source_width = self.source.z;
        let source_or_texture_width = if source_width > 0.0 {
            source_width
        } else {
            texture_size.x
        };

        let source_height = self.source.w;
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
    fn uvs(&self, texture_size: Vec2, use_half_pixel_offset: bool, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(
            texture_size,
            use_half_pixel_offset,
            &self.source,
            self.flip,
            uv,
            uv2,
        );
    }

    fn set_vertices<TVertex>(
        &self,
        texture_size: Vec2,
        use_half_pixel_offset: bool,
        use_indices: bool,
        vertex_offset: usize,
        vertices: &mut Vec<TVertex>,
    ) where
        TVertex: Clone + From<PosUvColor>,
    {
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
        self.uvs(texture_size, use_half_pixel_offset, &mut c1_uv, &mut c3_uv);
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

        if use_indices {
            vertices[vertex_offset] = c1;
            vertices[vertex_offset + 1] = c2;
            vertices[vertex_offset + 2] = c3;
            vertices[vertex_offset + 3] = c4;
        } else {
            vertices[vertex_offset] = c1.clone();
            vertices[vertex_offset + 1] = c2;
            vertices[vertex_offset + 2] = c3.clone();
            vertices[vertex_offset + 3] = c3;
            vertices[vertex_offset + 4] = c4;
            vertices[vertex_offset + 5] = c1;
        }
    }

    fn to_vertices<TVertex>(
        &self,
        texture_size: Vec2,
        use_half_pixel_offset: bool,
        use_indices: bool,
    ) -> Vec<TVertex>
    where
        TVertex: Clone + From<PosUvColor>,
    {
        let mut vertices = Vec::with_capacity(vertices_per_quad(use_indices) as usize);
        self.set_vertices(
            texture_size,
            use_half_pixel_offset,
            use_indices,
            0,
            &mut vertices,
        );
        vertices
    }
}

/// Represetns a standard quad with additional absolute scaling.
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
    pub fn new<TColor, TRect, TVec2>(
        position: TVec2,
        color: TColor,
        size: TVec2,
        source: TRect,
        flip: UvFlip,
    ) -> Self
    where
        TColor: Into<Color>,
        TRect: Into<Rectangle>,
        TVec2: Into<Vec2>,
    {
        Self {
            position: position.into(),
            color: color.into(),
            size: size.into(),
            source: source.into(),
            flip,
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
    fn uvs(&self, texture_size: Vec2, use_half_pixel_offset: bool, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(
            texture_size,
            use_half_pixel_offset,
            &self.source,
            self.flip,
            uv,
            uv2,
        );
    }
}

/// Represents a quad with fully customized draw.
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
    #[allow(clippy::too_many_arguments)]
    #[inline]
    #[must_use]
    pub fn new<TColor, TRect, TVec2>(
        position: TVec2,
        color: TColor,
        origin: TVec2,
        size: TVec2,
        scale: TVec2,
        rotation: f32,
        source: TRect,
        flip: UvFlip,
    ) -> Self
    where
        TColor: Into<Color>,
        TRect: Into<Rectangle>,
        TVec2: Into<Vec2>,
    {
        Self {
            position: position.into(),
            color: color.into(),
            origin: origin.into(),
            size: size.into(),
            scale: scale.into(),
            rotation,
            source: source.into(),
            flip,
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
    fn uvs(&self, texture_size: Vec2, use_half_pixel_offset: bool, uv: &mut Vec2, uv2: &mut Vec2) {
        calculate_uvs_with_source(
            texture_size,
            use_half_pixel_offset,
            &self.source,
            self.flip,
            uv,
            uv2,
        );
    }

    #[inline]
    fn get_color(&self) -> Color {
        self.color
    }
}

/// Calculates UVs with using OpenGL default left-to-right bottom-to-top texcoords by default, and
/// lets end users to flip UVs how they see fit with `flip` parameter.
pub(crate) fn calculate_uvs_with_source(
    texture_size: Vec2,
    use_half_pixel_offset: bool,
    source: &Rectangle,
    flip: UvFlip,
    uv: &mut Vec2,
    uv2: &mut Vec2,
) {
    if texture_size.x > 0.0 && texture_size.y > 0.0 {
        let (bottom, right) = if use_half_pixel_offset {
            (source.y + source.w - 1.0, source.x + source.z - 1.0)
        } else {
            (source.y + source.w, source.x + source.z)
        };
        let mut u = get_texel_coord(source.x, texture_size.x, use_half_pixel_offset);
        let mut v = get_texel_coord(bottom, texture_size.y, use_half_pixel_offset);
        let mut u2 = get_texel_coord(right, texture_size.x, use_half_pixel_offset);
        let mut v2 = get_texel_coord(source.y, texture_size.y, use_half_pixel_offset);
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
pub(crate) fn flip_uvs<'uvs, T>(
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
#[must_use]
#[inline]
pub(crate) fn make_vertices<TVertex>(
    color: Color,
    c1_position: Vec2,
    c2_position: Vec2,
    c3_position: Vec2,
    c4_position: Vec2,
    c1_uv: Vec2,
    c2_uv: Vec2,
    c3_uv: Vec2,
    c4_uv: Vec2,
) -> (TVertex, TVertex, TVertex, TVertex)
where
    TVertex: From<PosUvColor>,
{
    let c1 = TVertex::from(PosUvColor::new(c1_position, c1_uv, color));
    let c2 = TVertex::from(PosUvColor::new(c2_position, c2_uv, color));
    let c3 = TVertex::from(PosUvColor::new(c3_position, c3_uv, color));
    let c4 = TVertex::from(PosUvColor::new(c4_position, c4_uv, color));
    (c1, c2, c3, c4)
}

#[must_use]
#[inline]
pub(crate) fn get_texel_coord(v: f32, tex_dim: f32, use_half_pixel_offset: bool) -> f32 {
    if use_half_pixel_offset {
        (v + 0.5) / tex_dim
    } else {
        v / tex_dim
    }
}
