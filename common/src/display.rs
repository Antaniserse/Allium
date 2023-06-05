use std::borrow::Cow;
use std::error::Error;

use anyhow::Result;
use embedded_font::{FontTextStyle, FontTextStyleBuilder};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

use crate::constants::{BUTTON_DIAMETER, SELECTION_HEIGHT};
use crate::platform::{Color, Key};
use crate::stylesheet::Stylesheet;

pub trait Display<E: Error + Send + Sync + 'static>:
    OriginDimensions + DrawTarget<Color = Color, Error = E> + Sized
{
    fn map_pixels<F>(&mut self, f: F) -> Result<()>
    where
        F: FnMut(Color) -> Color;

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn save(&mut self) -> Result<()>;
    fn load(&mut self, area: Rectangle) -> Result<()>;

    fn darken(&mut self) -> Result<()> {
        self.map_pixels(|p| Color::new(p.r() / 4, p.g() / 4, p.b() / 4))?;
        self.flush()
    }

    fn draw_text(
        &mut self,
        point: Point,
        text: &str,
        style: FontTextStyle<Color>,
        alignment: Alignment,
    ) -> Result<Rectangle> {
        let text = Text::with_alignment(text, point, style, alignment);
        text.draw(self)?;
        Ok(text.bounding_box())
    }

    /// Truncated text to fit within width, adding ellipsis if truncated
    fn truncate_text_ellipsis<'a>(
        &self,
        point: Point,
        text: &'a str,
        style: FontTextStyle<Color>,
        alignment: Alignment,
        width: u32,
    ) -> Result<Cow<'a, str>> {
        let mut text = Text::with_alignment(text, point, style.clone(), alignment);
        let ellipsis_width = Text::with_alignment("...", point, style, alignment)
            .bounding_box()
            .size
            .width;

        // TODO: binary search?
        let mut ellipsis = false;
        let mut text_width = text.bounding_box().size.width;
        while text_width + ellipsis_width > width {
            let mut chars = text.text.chars();
            chars.next_back();
            ellipsis = true;
            text.text = chars.as_str();
            text_width = text.bounding_box().size.width;
        }
        let ellipsis_text = format!("{}...", text.text);
        Ok(if ellipsis {
            Cow::Owned(ellipsis_text)
        } else {
            Cow::Borrowed(text.text)
        })
    }

    fn draw_entry(
        &mut self,
        point: Point,
        text: &str,
        style: FontTextStyle<Color>,
        alignment: Alignment,
        width: u32,
        selected: bool,
    ) -> Result<Rectangle> {
        let Point { x, y } = point;

        let bg_color = style.background_color;
        let truncated_text = self.truncate_text_ellipsis(
            Point::new(x, y),
            text,
            style.clone(),
            Alignment::Left,
            width,
        )?;

        let text = Text::with_alignment(&truncated_text, point, style, alignment);
        let text_width = text.bounding_box().size.width;

        // Draw selection highlight
        if selected {
            if let Some(bg_color) = bg_color {
                let fill_style = PrimitiveStyle::with_fill(bg_color);
                Circle::new(Point::new(x - 12, y - 4), SELECTION_HEIGHT)
                    .into_styled(fill_style)
                    .draw(self)?;
                Circle::new(
                    Point::new(x + text_width as i32 - SELECTION_HEIGHT as i32 + 12, y - 4),
                    SELECTION_HEIGHT,
                )
                .into_styled(fill_style)
                .draw(self)?;
                Rectangle::new(
                    Point::new(x - 12 + SELECTION_HEIGHT as i32 / 2, y - 4),
                    Size::new(text_width - 24 + SELECTION_HEIGHT / 2, SELECTION_HEIGHT),
                )
                .into_styled(fill_style)
                .draw(self)?;
            }
        }

        // Draw text
        text.draw(self)?;

        Ok(Rectangle::new(
            Point::new(x - 12, y - 4),
            Size::new(text_width + 24, SELECTION_HEIGHT),
        ))
    }

    fn draw_button_hint(
        &mut self,
        point: Point,
        button: Key,
        style: FontTextStyle<Color>,
        text: &str,
        styles: &Stylesheet,
    ) -> Result<Rectangle> {
        let x = point.x
            - self
                .draw_text(
                    Point::new(point.x, point.y + 4),
                    text,
                    style,
                    Alignment::Right,
                )?
                .size
                .width as i32
            - 4;
        self.draw_button(
            Point::new(x - BUTTON_DIAMETER as i32, point.y),
            button,
            styles,
        )?;
        Ok(Rectangle::new(
            Point::new(x - BUTTON_DIAMETER as i32, point.y),
            Size::new(
                (point.x - (x - BUTTON_DIAMETER as i32)) as u32,
                BUTTON_DIAMETER,
            ),
        ))
    }

    fn draw_button(&mut self, point: Point, button: Key, styles: &Stylesheet) -> Result<()> {
        let (color, text) = match button {
            Key::A => (styles.button_a_color, "A"),
            Key::B => (styles.button_b_color, "B"),
            Key::X => (styles.button_x_color, "X"),
            Key::Y => (styles.button_y_color, "Y"),
            _ => (styles.primary, "?"),
        };

        Circle::new(point, BUTTON_DIAMETER)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(self)?;

        let button_style = FontTextStyleBuilder::new(styles.ui_font.clone())
            .font_size(styles.ui_font_size)
            .text_color(styles.fg_color)
            .background_color(color)
            .build();
        Text::with_text_style(
            text,
            Point::new(point.x + (BUTTON_DIAMETER / 2) as i32, point.y + 4),
            button_style,
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Middle)
                .build(),
        )
        .draw(self)?;

        Ok(())
    }
}