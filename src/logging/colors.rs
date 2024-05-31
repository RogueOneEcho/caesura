use colored::{ColoredString, Colorize, CustomColor};

struct Palette;
impl Palette {
    pub fn gray() -> CustomColor {
        const VALUE: u8 = 168;
        CustomColor::new(VALUE, VALUE, VALUE)
    }

    pub fn dark_gray() -> CustomColor {
        const VALUE: u8 = 112;
        CustomColor::new(VALUE, VALUE, VALUE)
    }
}

pub trait Colors {
    type Error;

    fn gray(&self) -> ColoredString;

    fn dark_gray(&self) -> ColoredString;
}

impl Colors for &str {
    type Error = ();

    fn gray(&self) -> ColoredString {
        self.custom_color(Palette::gray())
    }

    fn dark_gray(&self) -> ColoredString {
        self.custom_color(Palette::dark_gray())
    }
}

impl Colors for String {
    type Error = ();

    fn gray(&self) -> ColoredString {
        self.custom_color(Palette::gray())
    }

    fn dark_gray(&self) -> ColoredString {
        self.custom_color(Palette::dark_gray())
    }
}

impl Colors for ColoredString {
    type Error = ();

    fn gray(&self) -> ColoredString {
        self.clone().custom_color(Palette::gray())
    }

    fn dark_gray(&self) -> ColoredString {
        self.clone().custom_color(Palette::dark_gray())
    }
}
