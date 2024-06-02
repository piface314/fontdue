//! Performs basic text layout in Fontdue.

use fontdue::layout::{
    Block, CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, Span, VerticalAlign,
    WrapStyle,
};
use fontdue::{Font, FontSettings};

use std::fs::File;
use std::io::{self, Write};

// cargo run --example layout --release
pub fn main() -> io::Result<()> {
    // Read the font data.
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    // Parse it into the font type.
    let roboto_regular = Font::from_bytes(font, FontSettings::default()).unwrap();

    let h = if let Some(metrics) = roboto_regular.horizontal_line_metrics(35.0) {
        (metrics.ascent - metrics.descent) as usize
    } else {
        50
    };
    // The list of fonts that will be used during layout.
    // Create a layout context. Laying out text needs some heap allocations; reusing this context
    // reduces the need to reallocate space. We inform layout of which way the Y axis points here.
    let mut layout = Layout::new(&roboto_regular, 35.0, CoordinateSystem::PositiveYDown);
    // By default, layout is initialized with the default layout settings. This call is redundant, but
    // demonstrates setting the value with your custom settings.
    layout.reset(&LayoutSettings {
        horizontal_align: HorizontalAlign::Justify,
        vertical_align: VerticalAlign::Middle,
        max_width: Some(600.0),
        max_height: Some(600.0),
        wrap_style: WrapStyle::Word,
        ..LayoutSettings::default()
    });
    // The text that will be laid out, with inline blocks.
    layout.append(Span::text("Lorem ipsum dolor sit amet, consec tetur adipiscing elit. X", 0u8));
    layout.append(Span::text("super", 0u8).with_rise(24.0).with_px(24.0));
    layout.append(Span::text("sub ", 0u8).with_rise(-8.0).with_px(24.0));
    layout.append(Span::block(Block::new(30, h), 180u8));
    layout.append(Span::text(" Maecenas ac ornare erat.\nOrnare tristique tortor. ", 0u8));
    layout.append(Span::block(Block::new(50, h), 100u8));
    layout.append(Span::text(
        " Etiam sit amet neque in tellus commodo pretium. Nunc mattis nunc nec dictum faucibus.",
        0u8,
    ));
    layout.append(Span::text(" AVA", 0u8).with_kerning(-3.5));
    layout.finalize();
    render(&layout, File::create("layout.pgm").expect("file should be created"), 600, 600)
}

fn render<'a>(layout: &Layout<'a, u8>, mut file: File, w: usize, h: usize) -> io::Result<()> {
    file.write(format!("P5\n{} {}\n255\n", w, h).as_bytes())?;
    let mut bytes: Vec<u8> = vec![0; w * h];
    let glyphs = layout.glyphs();
    if let Some(lines) = layout.lines() {
        for line in lines.iter() {
            for glyph in &glyphs[line.glyph_start..=line.glyph_end] {
                if let Some(config) = glyph.key {
                    let font = glyph.font;
                    let (metrics, bitmap) = font.rasterize_config(config);

                    if metrics.width == 0 || glyph.char_data.is_whitespace() || metrics.height == 0 {
                        continue;
                    }

                    let x = (glyph.x) as i32;
                    let y = (glyph.y) as i32;

                    for (row, y) in bitmap.chunks_exact(metrics.width).zip(y..) {
                        for (value, x) in row.iter().zip(x..) {
                            let (x, y) = if x < 0 || y < 0 {
                                continue;
                            } else {
                                (x as usize, y as usize)
                            };

                            let value = *value;
                            if value == 0 {
                                continue;
                            }

                            bytes.get_mut(y * h + x).map(|b| *b = value);
                        }
                    }
                } else {
                    for dy in 0..glyph.height {
                        for dx in 0..glyph.width {
                            let x = glyph.x as usize + dx;
                            let y = glyph.y as usize + dy;
                            bytes.get_mut(y * h + x).map(|b| *b = glyph.user_data);
                        }
                    }
                }
            }
        }
    }
    file.write(&bytes[..])?;
    Ok(())
}
