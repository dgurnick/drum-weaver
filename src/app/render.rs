use log::info;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, Tabs},
};

use super::{ActiveFocus, App, MenuItem};

pub trait UiRenderTrait {
    fn render_ui(&mut self);
    fn render_menu(&mut self) -> Tabs<'static>;
    fn render_songs(&mut self) -> Table<'static>;
    fn render_queue(&mut self) -> Table<'static>;
    fn render_status(&mut self);
    fn render_devices(&mut self) -> Table<'static>;
}

impl UiRenderTrait for App {
    fn render_ui(&mut self) {
        if self.is_exiting {
            return;
        }

        let menu_view = self.render_menu();
        let songs_view = self.render_songs();
        let queue_view = self.render_queue();

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
                    MenuItem::Songs => {
                        let songlist_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                            .split(chunks[1]);

                        frame.render_stateful_widget(songs_view, songlist_chunks[0], &mut self.library_state);

                        frame.render_stateful_widget(queue_view, songlist_chunks[1], &mut self.queue_state);
                    }
                    MenuItem::Devices => {
                        //frame.render_widget(queue, chunks[1]);
                    }
                    MenuItem::Help => {
                        //frame.render_widget(queue, chunks[1]);
                    }
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
        todo!()
    }
}
