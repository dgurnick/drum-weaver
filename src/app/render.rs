use std::time::Duration;

use log::info;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, LineGauge, List, ListItem, Paragraph, Row, Table, Tabs},
};

use super::{devices::read_devices, ActiveFocus, App, MenuItem};

pub trait UiRenderTrait {
    fn render_ui(&mut self);
    fn render_menu(&mut self) -> Tabs<'static>;
    fn render_songs(&mut self) -> Table<'static>;
    fn render_queue(&mut self) -> Table<'static>;
    fn render_status(&mut self);
    fn render_devices(&mut self) -> Table<'static>;
    fn render_footer(&mut self) -> Paragraph<'static>;
    fn render_gauge(&mut self) -> LineGauge<'static>;
}

impl UiRenderTrait for App {
    fn render_ui(&mut self) {
        if self.is_exiting {
            return;
        }

        let menu_view = self.render_menu();
        let songs_view = if self.active_menu_item == MenuItem::Library { Some(self.render_songs()) } else { None };
        let queue_view = if self.active_menu_item == MenuItem::Library { Some(self.render_songs()) } else { None };
        let device_view = if self.active_menu_item == MenuItem::Devices { Some(self.render_devices()) } else { None };
        let footer_view = self.render_footer();
        let gauge_view = self.render_gauge();

        self.terminal
            .draw(|frame| {
                let size = frame.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(3), Constraint::Length(1)].as_ref())
                    .split(size);

                frame.render_widget(menu_view, chunks[0]);

                match self.active_menu_item {
                    MenuItem::Library => {
                        let songlist_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                            .split(chunks[1]);

                        frame.render_stateful_widget(songs_view.unwrap(), songlist_chunks[0], &mut self.library_state);

                        frame.render_stateful_widget(queue_view.unwrap(), songlist_chunks[1], &mut self.queue_state);
                    }
                    MenuItem::Devices => {
                        frame.render_stateful_widget(device_view.unwrap(), chunks[1], &mut self.device_state);
                    }
                    MenuItem::Help => {

                        //frame.render_widget(queue, chunks[1]);
                    }
                } // end match

                frame.render_widget(footer_view, chunks[2]);
                frame.render_widget(gauge_view, chunks[3]);
            })
            .expect("Unable to draw UI");
    }

    fn render_menu(&mut self) -> Tabs<'static> {
        let menu_titles = vec!["Songs", "Devices", "Help", "Quit"];

        let menu = menu_titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Line::from(vec![
                    Span::styled(first, Style::default().fg(Color::LightBlue).add_modifier(Modifier::UNDERLINED)),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        Tabs::new(menu)
            .select(self.active_menu_item.clone().into())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::LightBlue))
            //.highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"))
    }

    fn render_songs(&mut self) -> Table<'static> {
        let songlist_ui = if self.active_focus == ActiveFocus::Library {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(format!("Songs ({})", self.get_songs().len()))
                .border_type(BorderType::Thick)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .title(format!("Songs ({})", self.get_songs().len()))
                .border_type(BorderType::Plain)
        };

        let mut rows = vec![];
        for song in self.get_songs() {
            let mut is_selected = false;
            if let Some(active_track) = &self.active_track {
                if active_track.main_file.contains(&song.title) {
                    is_selected = true;
                }
            }

            let selected_fg = if is_selected {
                Color::LightBlue
            } else if self.active_focus == ActiveFocus::Library {
                Color::White
            } else {
                Color::Rgb(60, 60, 60)
            };

            let playlist_position = self
                .queue
                .iter()
                .position(|song_record| song_record.title == song.title.as_str())
                .map(|index| (index + 1).to_string())
                .unwrap_or_else(|| String::from(""));

            let playlist_cell = Cell::from(Span::styled(playlist_position, Style::default().fg(selected_fg)));

            let selected_cell = if is_selected {
                Cell::from(Span::styled("▶".to_string(), Style::default().fg(selected_fg)))
            } else {
                Cell::from(Span::styled("".to_string(), Style::default().fg(selected_fg)))
            };

            let row = Row::new(vec![
                playlist_cell,
                selected_cell,
                Cell::from(Span::styled(song.artist.clone(), Style::default().fg(selected_fg))),
                Cell::from(Span::styled(song.title.clone(), Style::default().fg(selected_fg))),
                Cell::from(Span::styled(song.album.clone(), Style::default().fg(selected_fg))),
                Cell::from(Span::styled(song.genre.clone(), Style::default().fg(selected_fg))),
                Cell::from(Span::styled(song.length.clone(), Style::default().fg(selected_fg))),
            ]);

            rows.push(row);
        }

        let highlight_style = if self.active_focus == ActiveFocus::Library {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else if self.active_focus == ActiveFocus::Queue {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::Black)
        };

        let song_table = Table::new(rows)
            .block(songlist_ui)
            .highlight_style(highlight_style)
            .header(Row::new(vec![
                Cell::from(Span::raw(" ")),
                Cell::from(Span::raw(" ")),
                Cell::from(Line::from(vec![
                    Span::styled("A", Style::default().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::BOLD)),
                    Span::styled("rtist", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("T", Style::default().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::BOLD)),
                    Span::styled("itle", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("A", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("l", Style::default().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::BOLD)),
                    Span::styled("bum", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("G", Style::default().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::BOLD)),
                    Span::styled("enre", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("D", Style::default().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::BOLD)),
                    Span::styled("uration", Style::default().add_modifier(Modifier::BOLD)),
                ])),
            ]))
            .widths(&[
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Length(10),
            ]);

        song_table
    }

    fn render_queue(&mut self) -> Table<'static> {
        let queue_ui = if self.active_focus == ActiveFocus::Queue {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(format!("Queue ({})", self.queue.len()))
                .border_type(BorderType::Thick)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .title(format!("Queue ({})", self.queue.len()))
                .border_type(BorderType::Plain)
        };

        let mut rows = vec![];
        let mut idx = 1;
        for song in self.queue.clone() {
            let mut is_selected = false;
            if let Some(active_track) = self.active_track.clone() {
                if active_track.main_file.contains(&song.file_name) {
                    is_selected = true;
                }
            }

            idx += 1;
            let selected_fg = if is_selected {
                Color::LightBlue
            } else if self.active_focus == ActiveFocus::Queue {
                Color::White
            } else {
                Color::Rgb(60, 60, 60)
            };

            let selected_cell = if is_selected {
                Cell::from(Span::styled("▶".to_string(), Style::default().fg(selected_fg)))
            } else {
                Cell::from(Span::styled("".to_string(), Style::default().fg(selected_fg)))
            };

            let row = Row::new(vec![
                selected_cell,
                Cell::from(Span::styled(song.title.clone(), Style::default().fg(selected_fg))),
                Cell::from(Span::styled(song.artist.clone(), Style::default().fg(selected_fg))),
            ]);

            rows.push(row);
        }

        let highlight_style = if self.active_focus == ActiveFocus::Queue {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else if self.active_focus == ActiveFocus::Library {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::Black)
        };

        let queue_table = Table::new(rows)
            .block(queue_ui)
            .highlight_style(highlight_style)
            .header(Row::new(vec![
                Cell::from(Span::styled("", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Song", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Artist", Style::default().add_modifier(Modifier::BOLD))),
            ]))
            .widths(&[Constraint::Length(1), Constraint::Percentage(45), Constraint::Percentage(45)]);

        queue_table
    }

    fn render_status(&mut self) {
        info!("rendering status");
    }

    fn render_devices(&mut self) -> Table<'static> {
        let device_ui = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Devices")
            .border_type(BorderType::Plain);

        let mut rows = vec![];
        let mut idx = 0;
        for device in read_devices() {
            let is_track = if self.track_device_idx == idx { "Yes" } else { "" };
            let is_click = if self.click_device_idx == idx { "Yes" } else { "" };

            let row = Row::new(vec![Cell::from(is_track), Cell::from(is_click), Cell::from(device.name.clone())]);
            rows.push(row);
            idx += 1;
        }

        let highlight_style = Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD);

        let device_table = Table::new(rows)
            .block(device_ui)
            .highlight_style(highlight_style)
            .header(Row::new(vec![
                Cell::from(Span::styled("Track?", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Click?", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Device", Style::default().add_modifier(Modifier::BOLD))),
            ]))
            .widths(&[Constraint::Length(10), Constraint::Length(10), Constraint::Percentage(45)]);

        device_table
    }

    fn render_footer(&mut self) -> Paragraph<'static> {
        let message = format!("{} | {}", self.player_status.as_string(), "OTHER STATUS");
        Paragraph::new(message).block(Block::default().borders(Borders::ALL))
    }

    fn render_gauge(&mut self) -> LineGauge<'static> {
        let start_color = (0, 255, 0);
        let end_color = (255, 0, 0);

        let (position, song_length) = self.current_position.unwrap_or((Duration::from_secs(0), Duration::from_secs(1)));

        // Calculate the progress ratio
        let progress_ratio = position.as_secs_f64() / song_length.as_secs_f64();

        let color = lerp_color(start_color, end_color, progress_ratio);

        LineGauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::White).add_modifier(Modifier::BOLD))
            .line_set(symbols::line::THICK)
            .ratio(progress_ratio)
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
