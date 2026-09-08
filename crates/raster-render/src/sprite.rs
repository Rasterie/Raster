use raster_math::{Rect, Vec2};

/// Which layer a sprite draws on.
///
/// Layers draw in ascending order, so a lower number is further back. Within a
/// layer, order is by insertion — a game that needs finer control uses more
/// layers rather than relying on submission order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Layer(pub i16);

impl Layer {
    pub const BACKGROUND: Self = Self(-100);
    pub const DEFAULT: Self = Self(0);
    pub const FOREGROUND: Self = Self(100);
    pub const UI: Self = Self(1000);
}

/// A colour with straight (non-premultiplied) alpha, in the 0..1 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Colour {
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Colour {
    fn default() -> Self {
        Self::WHITE
    }
}

/// One sprite to draw this frame.
///
/// Submitted to the renderer rather than stored: the batcher clears every
/// frame, so this describes an intent, not a resource.
#[derive(Debug, Clone, Copy)]
pub struct SpriteDraw {
    /// Top-left corner in world space.
    pub position: Vec2,
    /// Size in world pixels. Usually the source rectangle's size, but a sprite
    /// may be stretched.
    pub size: Vec2,
    /// The region of the texture to sample, in pixels.
    pub source: Rect,
    pub tint: Colour,
    pub layer: Layer,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl SpriteDraw {
    /// A sprite drawing the whole of a texture at its natural size.
    #[must_use]
    pub fn new(position: Vec2, texture_size: Vec2) -> Self {
        Self {
            position,
            size: texture_size,
            source: Rect::from_position_size(Vec2::ZERO, texture_size),
            tint: Colour::WHITE,
            layer: Layer::DEFAULT,
            flip_x: false,
            flip_y: false,
        }
    }

    /// The area this sprite covers in world space, for culling.
    #[must_use]
    pub fn bounds(self) -> Rect {
        Rect::from_position_size(self.position, self.size)
    }
}

/// What the GPU receives per sprite.
///
/// Mirrors the `Instance` struct in `sprite.wgsl`. `repr(C)` keeps the layout
/// predictable, which the vertex attribute offsets below depend on.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub tint: [f32; 4],
}

impl Instance {
    /// Builds the GPU form of a sprite.
    ///
    /// `atlas` is the texture's size in pixels, used to normalise the source
    /// rectangle into UV coordinates.
    pub(crate) fn new(draw: &SpriteDraw, atlas: Vec2) -> Self {
        let mut uv_min = Vec2::new(draw.source.left() / atlas.x, draw.source.top() / atlas.y);
        let mut uv_max = Vec2::new(
            draw.source.right() / atlas.x,
            draw.source.bottom() / atlas.y,
        );

        // Flipping swaps the UV bounds rather than the geometry: the quad's
        // winding stays the same, so no separate pipeline state is needed.
        if draw.flip_x {
            std::mem::swap(&mut uv_min.x, &mut uv_max.x);
        }
        if draw.flip_y {
            std::mem::swap(&mut uv_min.y, &mut uv_max.y);
        }

        Self {
            /*
              L'accrochage a la grille se fait ici, une fois, sur la position
              finale. Les sprites s'accrochent, la camera non : c'est ce couple
              qui donne des pixels nets et un defilement fluide, et l'inverser
              produit soit du flou soit des saccades.
            */
            position: draw.position.snap().to_array(),
            size: draw.size.to_array(),
            uv_min: uv_min.to_array(),
            uv_max: uv_max.to_array(),
            tint: draw.tint.to_array(),
        }
    }

    /// How the GPU reads the fields above.
    pub(crate) const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,  // position
        1 => Float32x2,  // size
        2 => Float32x2,  // uv_min
        3 => Float32x2,  // uv_max
        4 => Float32x4,  // tint
    ];

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
