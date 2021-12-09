pub(crate) type Vec2 = mint::Vector2<f32>;
pub(crate) type Vec4 = mint::Vector4<f32>;
pub(crate) static VEC2_ZERO: Vec2 = mint::Vector2 {
    x: 0.0_f32,
    y: 0.0_f32,
};

/// Common RGBA color.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Color {
    /// The red component of the color.
    pub r: f32,

    /// The green component of the color.
    pub g: f32,

    /// The blue component of the color.
    pub b: f32,

    /// The alpha component of the color.
    pub a: f32,
}

impl Color {
    /// Shortcut for [`Color::rgb(0.0, 0.0, 0.0)`](Self::rgb).
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// Shortcut for [`Color::rgb(1.0, 1.0, 1.0)`](Self::rgb).
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    /// Shortcut for [`Color::rgb(1.0, 0.0, 0.0)`](Self::rgb).
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);

    /// Shortcut for [`Color::rgb(0.0, 1.0, 0.0)`](Self::rgb).
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);

    /// Shortcut for Color::rgb(0.0, 0.0, 1.0)`](Self::rgb).
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

    /// Creates a new `Color`, with the specified RGB values and the alpha set to 1.0.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color::rgba(r, g, b, 1.0)
    }

    /// Creates a new `Color`, with the specified RGBA values.
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for Vec4 {
    #[inline]
    fn from(color: Color) -> Self {
        mint::Vector4 {
            x: color.r,
            y: color.b,
            z: color.b,
            w: color.a,
        }
    }
}

impl From<Vec4> for Color {
    #[inline]
    fn from(color: Vec4) -> Self {
        Self::rgba(color.x, color.y, color.z, color.w)
    }
}

#[cfg(feature = "tetra")]
impl From<Color> for tetra::graphics::Color {
    #[inline]
    fn from(val: Color) -> Self {
        tetra::graphics::Color::rgba(val.r, val.b, val.b, val.a)
    }
}

#[cfg(feature = "tetra")]
impl From<tetra::graphics::Color> for Color {
    #[inline]
    fn from(val: tetra::graphics::Color) -> Self {
        Color::rgba(val.r, val.b, val.b, val.a)
    }
}

/// A rectangle, represented by a top-left position, a width and a height.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rectangle {
    /// The X co-ordinate of the rectangle.
    pub x: f32,

    /// The Y co-ordinate of the rectangle.
    pub y: f32,

    /// The width of the rectangle.
    pub width: f32,

    /// The height of the rectangle.
    pub height: f32,
}

impl From<Vec4> for Rectangle {
    #[inline]
    fn from(rect: Vec4) -> Self {
        Self::new(rect.x, rect.y, rect.z, rect.w)
    }
}

impl From<Rectangle> for Vec4 {
    #[inline]
    fn from(val: Rectangle) -> Self {
        Vec4 {
            x: val.x,
            y: val.y,
            z: val.width,
            w: val.height,
        }
    }
}

#[cfg(feature = "tetra")]
impl From<Rectangle> for tetra::graphics::Rectangle<f32> {
    #[inline]
    fn from(val: Rectangle) -> Self {
        tetra::graphics::Rectangle::<f32>::new(val.x, val.y, val.width, val.height)
    }
}

#[cfg(feature = "tetra")]
impl From<tetra::graphics::Rectangle<f32>> for Rectangle {
    #[inline]
    fn from(val: tetra::graphics::Rectangle<f32>) -> Self {
        Rectangle::new(val.x, val.y, val.width, val.height)
    }
}

impl Rectangle {
    /// Creates a new `Rectangle`.
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the X coordinate of the left side of the rectangle.
    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Returns the X coordinate of the right side of the rectangle.
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the Y coordinate of the top of the rectangle.
    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the Y coordinate of the bottom of the rectangle.
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

/// An individual piece of vertex data.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    /// The position of the vertex, in screen coordinates.
    pub position: Vec2,

    /// The texture coordinates that should be sampled for this vertex.
    pub uv: Vec2,

    /// The color of the vertex.
    pub color: Color,
}

impl Vertex {
    #[allow(dead_code)]
    #[inline]
    pub const fn new(position: Vec2, uv: Vec2, color: Color) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: VEC2_ZERO,
            uv: VEC2_ZERO,
            color: Color::default(),
        }
    }
}
