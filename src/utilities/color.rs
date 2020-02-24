#[allow(dead_code)]
#[derive(Debug, PartialEq, Copy, Serialize, Deserialize, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[allow(dead_code)]
    pub fn new(r: f32, b: f32, g: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    #[allow(dead_code)]
    pub fn with_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: Color::into_linear(r as f32 / 255.0),
            g: Color::into_linear(g as f32 / 255.0),
            b: Color::into_linear(b as f32 / 255.0),
            a: a as f32 / 255.0,
        }
    }

    #[allow(dead_code)]
    pub fn into_raw_u32(self) -> [u32; 4] {
        [
            self.r.to_bits(),
            self.g.to_bits(),
            self.b.to_bits(),
            self.a.to_bits(),
        ]
    }

    pub fn into_linear(number: f32) -> f32 {
        number.powf(2.2)
    }

    pub fn into_linear_multiple(numbers: &[f32; 4]) -> [f32; 4] {
        let mut ret = [0.0; 4];
        for this_number in 0..3 {
            ret[this_number] = Self::into_linear(numbers[this_number]);
        }
        ret[3] = numbers[3];
        ret
    }

    pub fn inspect(&mut self, ui: &imgui::Ui<'_>, label: &str, uid: &str) {
        let mut our_color: [f32; 4] = self.clone().into();
        if imgui::ColorEdit::new(&imgui::im_str!("{}##{}", label, uid), &mut our_color).build(ui) {
            *self = our_color.into();
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(w: [f32; 4]) -> Color {
        Color {
            r: w[0],
            g: w[1],
            b: w[2],
            a: w[3],
        }
    }
}
impl From<Color> for [f32; 4] {
    fn from(o: Color) -> [f32; 4] {
        [o.r, o.g, o.b, o.a]
    }
}
