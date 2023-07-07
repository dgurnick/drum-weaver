use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Widget};

pub struct CustomGauge<'a> {
    value: f64,
    max_value: f64,
    style: Style,
    block: Option<Block<'a>>,
}

impl<'a> CustomGauge<'a> {
    pub fn new(value: f64, max_value: f64, style: Style) -> CustomGauge<'a> {
        CustomGauge { value, max_value, style, block: None }
    }
}

impl<'a> Widget for CustomGauge<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let gauge_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if gauge_area.height < 2 {
            return;
        }

        let start_color = (0, 255, 0);
        let end_color = (255, 0, 0);
        let color = lerp_color(start_color, end_color, self.value / self.max_value);
        let mut style = self.style;
        style.fg = color.into();

        let gauge_width = ((gauge_area.width as f64) * (self.value / self.max_value)) as u16;

        for x in gauge_area.left()..gauge_width {
            buf.get_mut(x, gauge_area.top()).set_symbol("█").set_style(style);
            buf.get_mut(x, gauge_area.top() + 1).set_symbol("█").set_style(style);
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

impl<'a> CustomGauge<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
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
