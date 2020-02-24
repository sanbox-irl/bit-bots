use super::{imgui_system, math, vec_iter::*, Axis, Vec2Int};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn with_single(scalar: f32) -> Self {
        Vec2 { x: scalar, y: scalar }
    }

    pub fn with_axis(scalar: f32, axis: Axis) -> Self {
        match axis {
            Axis::X => Self { x: scalar, y: 0.0 },
            Axis::Y => Self { x: 0.0, y: scalar },
        }
    }

    pub fn with_plus_x(o: Self, x: f32) -> Self {
        Self { x: o.x + x, y: o.y }
    }

    pub fn with_plus_y(o: Self, y: f32) -> Self {
        Self { x: o.x, y: o.y + y }
    }

    pub fn with_plus_axis(o: Self, scalar: f32, axis: Axis) -> Self {
        match axis {
            Axis::X => Self::with_plus_x(o, scalar),
            Axis::Y => Self::with_plus_y(o, scalar),
        }
    }

    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalize(&mut self) {
        let m = self.magnitude();
        self.x = self.x / m;
        self.y = self.y / m;
    }

    pub fn floored(&mut self) {
        self.x = self.x.floor();
        self.y = self.y.floor();
    }

    pub fn ceiled(&mut self) {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
    }

    pub fn round(&mut self) {
        self.x = self.x.round();
        self.y = self.y.round();
    }

    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    pub fn ceil(&mut self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    pub fn rounded(&mut self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    pub fn reflected(&mut self, axis: Axis) {
        match axis {
            Axis::X => self.x *= -1.0,
            Axis::Y => self.y *= -1.0,
        };
    }

    pub fn reflect(&self, axis: Axis) -> Self {
        let mut ret = self.clone();
        match axis {
            Axis::X => ret.x *= -1.0,
            Axis::Y => ret.y *= -1.0,
        };

        ret
    }

    pub fn asymptotic_move(&self, other_vec: Vec2, weight: f32) -> Self {
        Vec2::new(
            math::asymptotic_motion(self.x, other_vec.x, weight),
            math::asymptotic_motion(self.y, other_vec.y, weight),
        )
    }

    pub fn asymptotic_moved(&mut self, other_vec: Vec2, weight: f32) {
        self.x = math::asymptotic_motion(self.x, other_vec.x, weight);
        self.y = math::asymptotic_motion(self.y, other_vec.y, weight);
    }

    pub fn approach(&self, dest: Vec2, speed: Vec2) -> Self {
        Vec2::new(
            math::approach(self.x, dest.x, speed.x),
            math::approach(self.y, dest.y, speed.y),
        )
    }

    pub fn approached(&mut self, dest: Vec2, speed: Vec2) {
        self.x = math::approach(self.x, dest.x, speed.x);
        self.y = math::approach(self.y, dest.y, speed.y);
    }

    pub fn get_axis_vec2(&self, axis: Axis) -> Self {
        match axis {
            Axis::X => Vec2::new(self.x, 0.0),
            Axis::Y => Vec2::new(0.0, self.y),
        }
    }

    pub fn get_axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    pub fn add_axis(&mut self, scalar: f32, axis: Axis) {
        match axis {
            Axis::X => self.x += scalar,
            Axis::Y => self.y += scalar,
        }
    }

    pub fn cwise_product(&self, other_vec: Vec2) -> Vec2 {
        let mut me: Vec2 = self.clone();
        me.x *= other_vec.x;
        me.y *= other_vec.y;
        me
    }

    pub fn cwise_div(&self, other_vec: Vec2) -> Vec2 {
        let mut me: Vec2 = self.clone();
        me.x /= other_vec.x;
        me.y /= other_vec.y;
        me
    }

    pub fn cwise_addition(&self, other_vec: Vec2) -> Vec2 {
        let mut me = self.clone();
        me.x += other_vec.x;
        me.y += other_vec.y;
        me
    }

    pub fn cwise_subtraction(&self, other_vec: Vec2) -> Vec2 {
        let mut me = self.clone();
        me.x -= other_vec.x;
        me.y -= other_vec.y;
        me
    }

    pub fn normalized(&self) -> Self {
        let m = self.magnitude();
        self.clone() / m
    }

    pub fn into_raw_usize(self) -> Result<(usize, usize), &'static str> {
        if self.x < 0.0 || self.y < 0.0 {
            Err("This is a negative number! Cannot case to usize intelligently.")
        } else {
            Ok((self.x as usize, self.y as usize))
        }
    }

    pub fn to_bits(self) -> [u32; 2] {
        [self.x.to_bits(), self.y.to_bits()]
    }

    pub fn clamp_components(&mut self, min_vec: &Vec2, max_vec: &Vec2) {
        self.x = self.x.max(min_vec.x).min(max_vec.x);

        self.y = self.y.max(min_vec.y).min(max_vec.y);
    }
}

impl Vec2 {
    #[allow(dead_code)]
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    #[allow(dead_code)]
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    #[allow(dead_code)]
    pub const UP: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    #[allow(dead_code)]
    pub const RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(w: [f32; 2]) -> Vec2 {
        Vec2 { x: w[0], y: w[1] }
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(w: Vec2) -> [f32; 2] {
        [w.x, w.y]
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(w: Vec2) -> (f32, f32) {
        (w.x, w.y)
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(w: (f32, f32)) -> Vec2 {
        Vec2::new(w.0, w.1)
    }
}

impl From<Vec2Int> for Vec2 {
    fn from(other: Vec2Int) -> Vec2 {
        Vec2 {
            x: other.x as f32,
            y: other.y as f32,
        }
    }
}

impl From<winit::dpi::LogicalSize<f64>> for Vec2 {
    fn from(other: winit::dpi::LogicalSize<f64>) -> Vec2 {
        Vec2::new(other.width as f32, other.height as f32)
    }
}

impl GameVec<f32> for Vec2 {
    fn x(&self) -> &f32 {
        &self.x
    }

    fn y(&self) -> &f32 {
        &self.y
    }
}

impl Vec2 {
    pub fn iter(&self) -> VecIter<'_, f32> {
        VecIter::new(self)
    }
}

use imgui;
impl Vec2 {
    pub fn inspector(&mut self, ui: &imgui::Ui<'_>, label: &imgui::ImStr) -> bool {
        let mut vec2_deconstructed = self.clone().into();

        if ui.input_float2(label, &mut vec2_deconstructed).build() {
            self.x = vec2_deconstructed[0];
            self.y = vec2_deconstructed[1];
            true
        } else {
            false
        }
    }

    pub fn no_interact_inspector(&mut self, ui: &imgui::Ui<'_>, label: &imgui::ImStr) -> bool {
        let mut vec2_deconstructed = self.clone().into();
        let mut return_value = false;

        imgui_system::wrap_style_var(ui, imgui::StyleVar::Alpha(0.3), || {
            if ui
                .input_float2(label, &mut vec2_deconstructed)
                .flags(imgui::ImGuiInputTextFlags::ReadOnly)
                .build()
            {
                self.x = vec2_deconstructed[0];
                self.y = vec2_deconstructed[1];
                return_value = true
            }
        });

        return_value
    }
}
