/// The camera: which part of the scene is visible.
///
/// It doesn't depend on the frame's resolution, so changing the resolution
/// only changes sharpness, never what is shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Magnification. At 1, the frame's shorter side spans scene
    /// coordinates -1 to 1; at 2 it spans -0.5 to 0.5.
    pub zoom: f64,
}

impl Default for View {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

impl View {
    /// Returns the mapping between scene coordinates and the pixels of a
    /// frame of the given size.
    pub fn mapping(&self, width: usize, height: usize) -> Mapping {
        let (w, h) = (width as f64, height as f64);
        Mapping {
            cx: w / 2.0,
            cy: h / 2.0,
            unit: w.min(h) / 2.0 * self.zoom,
        }
    }
}

/// Converts between pixel positions in one frame and scene coordinates.
///
/// Scene coordinates have their origin at the frame centre, with x to the
/// right and y down. Scenes go through this type instead of computing
/// positions from pixel indices, so that pixel layouts other than a square
/// grid can be added here later.
#[derive(Clone, Copy, Debug)]
pub struct Mapping {
    /// Frame centre, in pixels.
    cx: f64,
    cy: f64,
    /// Pixels per scene unit.
    unit: f64,
}

impl Mapping {
    /// Returns the scene coordinates of the centre of pixel (x, y).
    pub fn to_scene(self, x: usize, y: usize) -> (f64, f64) {
        (
            (x as f64 + 0.5 - self.cx) / self.unit,
            (y as f64 + 0.5 - self.cy) / self.unit,
        )
    }

    /// Returns the position of a scene point, in fractional pixels.
    pub fn to_pixel(self, (x, y): (f64, f64)) -> (f64, f64) {
        (x * self.unit + self.cx, y * self.unit + self.cy)
    }

    /// Returns the number of pixels per scene unit.
    pub fn unit(&self) -> f64 {
        self.unit
    }
}
