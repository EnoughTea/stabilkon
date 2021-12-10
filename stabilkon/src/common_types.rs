pub(crate) type Color = Vec4;
pub(crate) type Rectangle = Vec4;
pub(crate) type Vec2 = mint::Vector2<f32>;
pub(crate) type Vec4 = mint::Vector4<f32>;

pub(crate) static VEC2_ZERO: Vec2 = mint::Vector2 {
    x: 0.0_f32,
    y: 0.0_f32,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PosUvColor {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

impl PosUvColor {
    #[inline]
    pub fn new<TColor, TVec2>(position: TVec2, uv: TVec2, color: TColor) -> Self
    where
        TColor: Into<Color>,
        TVec2: Into<Vec2>,
    {
        Self {
            position: position.into(),
            uv: uv.into(),
            color: color.into(),
        }
    }
}

#[cfg(feature = "ggez")]
impl From<PosUvColor> for ggez::graphics::Vertex {
    fn from(color_pos_uv: PosUvColor) -> Self {
        Self {
            pos: color_pos_uv.position.into(),
            uv: color_pos_uv.uv.into(),
            color: color_pos_uv.color.into(),
        }
    }
}

#[cfg(feature = "tetra")]
impl From<PosUvColor> for tetra::graphics::mesh::Vertex {
    fn from(color_pos_uv: PosUvColor) -> Self {
        Self::new(
            tetra::math::Vec2::new(color_pos_uv.position.x, color_pos_uv.position.y),
            tetra::math::Vec2::new(color_pos_uv.uv.x, color_pos_uv.uv.y),
            tetra::graphics::Color::rgba(
                color_pos_uv.color.x,
                color_pos_uv.color.y,
                color_pos_uv.color.z,
                color_pos_uv.color.w,
            ),
        )
    }
}
