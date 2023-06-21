use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use super::App;

pub trait UiRenderTrait {
    fn render_ui(&mut self);
}

impl UiRenderTrait for App {
    fn render_ui(&mut self) {
        // render UI elements
        self.terminal
            .draw(|frame| {
                let size = frame.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(3), Constraint::Length(1)].as_ref())
                    .split(size);

                let block = Block::default().title("Block").borders(Borders::ALL);
                frame.render_widget(block, size);
            })
            .expect("Unable to draw UI");
    }
}
