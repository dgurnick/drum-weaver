use std::time::Duration;

use log::info;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Table, Tabs},
};

use super::{devices::read_devices, player::PlaybackStatus, status_bar::CustomGauge, ActiveFocus, App, MenuItem, PlayerStatus};

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
        let queue_view = if self.active_menu_item == MenuItem::Library { Some(self.render_queue()) } else { None };
        let device_view = if self.active_menu_item == MenuItem::Devices { Some(self.render_devices()) } else { None };
        let footer_view = self.render_footer();
        //let gauge_view = self.render_gauge();

        let gauge_view = match &self.playback_status {
            Some(status) => {
                if status.track_position.is_some() && status.track_duration.is_some() {
                    let gauge_view = CustomGauge::new(
                        status.track_position.unwrap().as_secs() as f64,
                        status.track_duration.unwrap().as_secs() as f64,
                        Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD),
                    );

                    Some(gauge_view)
                } else {
                    None
                }
            }
            None => None,
        };

        self.terminal
            .draw(|frame| {
                let constraints = match self.player_status {
                    PlayerStatus::Playing(_) => [Constraint::Length(3), Constraint::Min(3), Constraint::Length(3), Constraint::Length(2)].as_ref(),
                    _ => [Constraint::Length(3), Constraint::Min(3), Constraint::Length(3)].as_ref(),
                };

                let size = frame.size();
                let chunks = Layout::default().direction(Direction::Vertical).margin(2).constraints(constraints).split(size);

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
                match self.player_status {
                    PlayerStatus::Playing(_) => {
                        if let Some(gauge_view) = gauge_view {
                            frame.render_widget(gauge_view, chunks[3]);
                        }
                    }
                    _ => {}
                }
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
            .block(Block::default().title("Menu").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::LightBlue))
            //.highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"))
    }

    fn render_songs(&mut self) -> Table<'static> {
        let songlist_ui = if self.active_focus == ActiveFocus::Library {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Yellow))
                .title(format!("Songs ({})", self.library.as_ref().unwrap().get_songs().len()))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .title(format!("Songs ({})", self.library.as_ref().unwrap().get_songs().len()))
        };

        let mut rows = vec![];
        for song in self.library.as_ref().unwrap().get_songs() {
            let mut is_selected = false;
            if let Some(active_stub) = &self.active_stub {
                info!("active_track: {:?}", active_stub);
                if active_stub.file_name == song.file_name {
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
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Yellow))
                .title(format!("Queue ({})", self.queue.len()))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .title(format!("Queue ({})", self.queue.len()))
        };

        let mut rows = vec![];
        for song in self.queue.clone() {
            let mut is_selected = false;
            if let Some(stub) = self.active_stub.clone() {
                if stub.file_name == song.file_name {
                    is_selected = true;
                }
            }

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
        let mut status = vec![];
        status.push(Span::styled(self.player_status.as_string().clone(), Style::default().fg(Color::LightBlue)));

        match self.player_status {
            PlayerStatus::Playing(_) => {
                let stub = self.active_stub.as_ref().unwrap();
                status.push(Span::raw(": "));
                status.push(Span::styled(stub.title.clone(), Style::default().add_modifier(Modifier::UNDERLINED)));
                status.push(Span::raw(" by "));
                status.push(Span::styled(stub.artist.clone(), Style::default().add_modifier(Modifier::UNDERLINED)));
            }
            PlayerStatus::Paused => {
                let stub = self.active_stub.as_ref().unwrap();
                status.push(Span::raw(": "));
                status.push(Span::styled(stub.title.clone(), Style::default().add_modifier(Modifier::UNDERLINED)));
                status.push(Span::raw(" by "));
                status.push(Span::styled(stub.artist.clone(), Style::default().add_modifier(Modifier::UNDERLINED)));
            }
            _ => {}
        }

        status.push(Span::raw(" | "));
        status.push(Span::styled("Track Volume: ", Style::default().fg(Color::LightBlue)));
        status.push(Span::raw(self.track_volume.to_string()));
        status.push(Span::styled(" Click Volume: ", Style::default().fg(Color::LightBlue)));
        status.push(Span::raw(self.click_volume.to_string()));

        let spans = Line::from(status);

        Paragraph::new(spans).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
    }

    fn render_gauge(&mut self) -> LineGauge<'static> {
        let start_color = (0, 255, 0);
        let end_color = (255, 0, 0);

        let track_position = self
            .playback_status
            .as_ref()
            .and_then(|status| status.track_position) // If PlaybackStatus exists, get track_position
            .unwrap_or(Duration::from_secs(0)); // If it is None, default to 0 seconds

        let track_duration = self
            .playback_status
            .as_ref()
            .and_then(|status| status.track_duration) // If PlaybackStatus exists, get track_duration
            .unwrap_or(Duration::from_secs(0)); // If it is None, default to 0 seconds

        // Calculate the progress ratio
        let progress_ratio = track_position.as_secs_f64() / track_duration.as_secs_f64();

        let color = lerp_color(start_color, end_color, progress_ratio);

        let gauge = LineGauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::White).add_modifier(Modifier::BOLD))
            .line_set(symbols::line::THICK);
        //.ratio(progress_ratio)

        if track_duration.as_secs() > 0 {
            gauge.ratio(progress_ratio)
        } else {
            gauge
        }
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
