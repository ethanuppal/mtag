// Copyright (C) 2025 Ethan Uppal. All rights reserved.

use ratatui::crossterm::style::{
    Attribute, Attributes, Color, Colors, ContentStyle,
};

fn color_shim(inquire_color: inquire::ui::Color) -> Color {
    match inquire_color {
        inquire::ui::Color::Black => Color::Black,
        inquire::ui::Color::LightRed => Color::Red,
        inquire::ui::Color::DarkRed => Color::DarkRed,
        inquire::ui::Color::LightGreen => Color::Green,
        inquire::ui::Color::DarkGreen => Color::DarkGreen,
        inquire::ui::Color::LightYellow => Color::Yellow,
        inquire::ui::Color::DarkYellow => Color::DarkYellow,
        inquire::ui::Color::LightBlue => Color::Blue,
        inquire::ui::Color::DarkBlue => Color::DarkBlue,
        inquire::ui::Color::LightMagenta => Color::Magenta,
        inquire::ui::Color::DarkMagenta => Color::DarkMagenta,
        inquire::ui::Color::LightCyan => Color::Cyan,
        inquire::ui::Color::DarkCyan => Color::DarkCyan,
        inquire::ui::Color::White => Color::White,
        inquire::ui::Color::Grey => Color::Grey,
        inquire::ui::Color::DarkGrey => Color::DarkGrey,
        inquire::ui::Color::Rgb { r, g, b } => Color::Rgb { r, g, b },
        inquire::ui::Color::AnsiValue(value) => Color::AnsiValue(value),
    }
}

fn attributes_shim(inquire_attributes: inquire::ui::Attributes) -> Attributes {
    let mut result = Attributes::none();
    if inquire_attributes.contains(inquire::ui::Attributes::BOLD) {
        result.set(Attribute::Bold);
    }
    if inquire_attributes.contains(inquire::ui::Attributes::ITALIC) {
        result.set(Attribute::Italic);
    }
    result
}

pub fn stylesheet_shim(stylesheet: inquire::ui::StyleSheet) -> ContentStyle {
    ContentStyle {
        foreground_color: stylesheet.fg.map(color_shim),
        background_color: stylesheet.bg.map(color_shim),
        underline_color: None,
        attributes: attributes_shim(stylesheet.att),
    }
}
