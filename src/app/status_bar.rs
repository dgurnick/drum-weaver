use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct CustomGauge {
    value: f64,
    max_value: f64,
    style: Style,
}

impl CustomGauge {
    pub fn new(value: f64, max_value: f64, style: Style) -> CustomGauge {
        CustomGauge { value, max_value, style }
    }
}

impl Widget for CustomGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let start_color = (0, 255, 0);
        let end_color = (255, 0, 0);
        let color = lerp_color(start_color, end_color, self.value / self.max_value);
        let mut style = self.style;
        style.fg = color.into();

        let gauge_width = ((area.width as f64) * (self.value / self.max_value)) as u16;

        for x in area.left()..gauge_width {
            buf.get_mut(x, area.top()).set_symbol("█").set_style(style);
            buf.get_mut(x, area.top() + 1).set_symbol("█").set_style(style);
        }

        // let pos_x = (area.width / 2) as u16;
        // let pos_y = area.top();

        // // Format the percentage to a string
        // let percentage = (self.value / self.max_value * 100.0).ceil() as i32;

        // let percentage_str = format!("{:.0}%", percentage);

        // // Write the percentage to the buffer
        // buf.set_string(pos_x, pos_y, &percentage_str, style.bg(color.into()).fg(Color::Black));
    }
}

// Function to perform linear interpolation (lerp) for colors
fn lerp_color(start_color: (u8, u8, u8), end_color: (u8, u8, u8), t: f64) -> Color {
    let (start_r, start_g, start_b) = start_color;
    let (end_r, end_g, end_b) = end_color;

    let r = lerp(start_r, end_r, t);
    let g = lerp(start_g, end_g, t);
    let b = lerp(start_b, end_b, t);

    Color::Rgb(r, g, b)
}

fn lerp(start: u8, end: u8, t: f64) -> u8 {
    (start as f64 + (end as f64 - start as f64) * t) as u8
}
