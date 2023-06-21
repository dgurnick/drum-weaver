use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use super::{ActiveFocus, App};

#[rustfmt::skip]
pub trait Render {
    fn render_songs<'a>(&mut self, is_playing: bool, ) -> Table<'a>;
    fn render_queue<'a>(&mut self, is_playing: bool, ) -> Table<'a>;
    fn render_help<'a>(&self) -> Paragraph<'a>;
    fn render_devices<'a>(&mut self, track_device: usize, click_device: usize) -> List<'a>;
}

#[rustfmt::enable]
impl Render for App {
    fn render_songs<'a>(&mut self, is_playing: bool) -> Table<'a> {
        let songlist_ui = if self.active_focus == ActiveFocus::Songs {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(format!("Songs ({})", self.songs.len()))
                .border_type(BorderType::Thick)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .title(format!("Songs ({})", self.songs.len()))
                .border_type(BorderType::Plain)
        };

        let _selected_song = self.songs.get(
            self.songlist_state
                .selected()
                .expect("there is always a selected song"),
        );

        let _selected_playlist_song = self.queue.get(self.active_queue_idx).cloned();

        let mut rows = vec![];
        for song in self.songs.clone() {
            let mut is_selected = false;
            if is_playing {
                if let Some(track_file) = self.track_file.clone() {
                    if track_file.contains(&song.title) {
                        is_selected = true;
                    }
                }
            }

            let selected_fg = if is_selected {
                Color::LightBlue
            } else {
                if self.active_focus == ActiveFocus::Songs {
                    Color::White
                } else {
                    Color::Rgb(60, 60, 60)
                }
            };

            let playlist_position = self
                .queue
                .iter()
                .position(|song_record| song_record.title == song.title.as_str())
                .map(|index| (index + 1).to_string())
                .unwrap_or_else(|| String::from(""));

            let playlist_cell = Cell::from(Span::styled(
                playlist_position,
                Style::default().fg(selected_fg),
            ));

            let selected_cell = if is_selected {
                Cell::from(Span::styled(
                    "▶".to_string(),
                    Style::default().fg(selected_fg),
                ))
            } else {
                Cell::from(Span::styled(
                    "".to_string(),
                    Style::default().fg(selected_fg),
                ))
            };

            let row = Row::new(vec![
                playlist_cell,
                selected_cell,
                Cell::from(Span::styled(
                    song.artist.clone(),
                    Style::default().fg(selected_fg),
                )),
                Cell::from(Span::styled(
                    song.title.clone(),
                    Style::default().fg(selected_fg),
                )),
                Cell::from(Span::styled(
                    song.album.clone(),
                    Style::default().fg(selected_fg),
                )),
                Cell::from(Span::styled(
                    song.genre.clone(),
                    Style::default().fg(selected_fg),
                )),
                Cell::from(Span::styled(
                    song.length.clone(),
                    Style::default().fg(selected_fg),
                )),
            ]);

            rows.push(row);
        }

        let highlight_style = if self.active_focus == ActiveFocus::Songs {
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            if self.active_focus == ActiveFocus::Songs {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::Black)
            }
        };

        let song_table = Table::new(rows)
            .block(songlist_ui)
            .highlight_style(highlight_style)
            .header(Row::new(vec![
                Cell::from(Span::raw(" ")),
                Cell::from(Span::raw(" ")),
                Cell::from(Line::from(vec![
                    Span::styled(
                        "A",
                        Style::default()
                            .add_modifier(Modifier::UNDERLINED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("rtist", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled(
                        "T",
                        Style::default()
                            .add_modifier(Modifier::UNDERLINED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("itle", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("A", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        "l",
                        Style::default()
                            .add_modifier(Modifier::UNDERLINED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("bum", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled(
                        "G",
                        Style::default()
                            .add_modifier(Modifier::UNDERLINED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("enre", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled(
                        "D",
                        Style::default()
                            .add_modifier(Modifier::UNDERLINED)
                            .add_modifier(Modifier::BOLD),
                    ),
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

    fn render_queue<'a>(&mut self, is_playing: bool) -> Table<'a> {
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
            if is_playing {
                if let Some(track_file) = self.track_file.clone() {
                    if track_file.contains(&song.file_name) {
                        is_selected = true;
                        self.active_queue_idx = idx;
                    }
                }
            }
            idx += 1;
            let selected_fg = if is_selected {
                Color::LightBlue
            } else {
                if self.active_focus == ActiveFocus::Queue {
                    Color::White
                } else {
                    Color::Rgb(60, 60, 60)
                }
            };

            let selected_cell = if is_selected {
                Cell::from(Span::styled(
                    "▶".to_string(),
                    Style::default().fg(selected_fg),
                ))
            } else {
                Cell::from(Span::styled(
                    "".to_string(),
                    Style::default().fg(selected_fg),
                ))
            };

            let row = Row::new(vec![
                selected_cell,
                Cell::from(Span::styled(
                    song.title.clone(),
                    Style::default().fg(selected_fg),
                )),
                Cell::from(Span::styled(
                    song.artist.clone(),
                    Style::default().fg(selected_fg),
                )),
            ]);

            rows.push(row);
        }

        let highlight_style = if self.active_focus == ActiveFocus::Queue {
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            if self.active_focus == ActiveFocus::Songs {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::Black)
            }
        };

        let queue_table = Table::new(rows)
            .block(queue_ui)
            .highlight_style(highlight_style)
            .header(Row::new(vec![
                Cell::from(Span::styled(
                    "",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Song",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Artist",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ]))
            .widths(&[
                Constraint::Length(1),
                Constraint::Percentage(45),
                Constraint::Percentage(45),
            ]);

        queue_table
    }

    fn render_help<'a>(&self) -> Paragraph<'a> {
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow);

        let help_content = vec![
            Line::from(Span::styled("General Commands", header_style)),
            Line::from(vec![Span::styled("q", Style::default().fg(Color::LightCyan)), Span::raw(": Quit (boo!).")]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::LightCyan)),
                Span::raw(": Show device selection. You can pick output devices for tracks and clicks separately."),
            ]),
            Line::from(vec![Span::styled("s", Style::default().fg(Color::LightCyan)), Span::raw(": Show the song list.")]),
            Line::from(vec![Span::styled("h", Style::default().fg(Color::LightCyan)), Span::raw(": Show this help screen.")]),
            Line::from("\n"),
            Line::from(Span::styled("Song list Commands", header_style)),
            Line::from(vec![
                Span::styled("n", Style::default().fg(Color::LightCyan)),
                Span::raw(": Move to the next song in the queue or song list."),
            ]),
            Line::from(vec![
                Span::styled("Left or Right Arrow", Style::default().fg(Color::LightCyan)),
                Span::raw(": Slow down or speed up playback."),
            ]),
            Line::from(vec![Span::styled("r", Style::default().fg(Color::LightCyan)), Span::raw(": Reset the playback speed.")]),
            Line::from(vec![Span::styled("z", Style::default().fg(Color::LightCyan)), Span::raw(": Restart the song that is playing.")]),
            Line::from(vec![
                Span::styled("SPACE", Style::default().fg(Color::LightCyan)),
                Span::raw(": Pause or continue the song that is playing"),
            ]),
            Line::from(vec![
                Span::styled("HOME or END", Style::default().fg(Color::LightCyan)),
                Span::raw(": Move to the first or last song in the queue or song list."),
            ]),
            Line::from(vec![Span::styled("x", Style::default().fg(Color::LightCyan)), Span::raw(": Shuffle or un-shuffle the playlist")]),
            Line::from(vec![Span::styled("1 or 4", Style::default().fg(Color::LightCyan)), Span::raw(": Lower the track or click volume")]),
            Line::from(vec![Span::styled("2 or 5", Style::default().fg(Color::LightCyan)), Span::raw(": Reset the track or click volume")]),
            Line::from(vec![Span::styled("3 or 6", Style::default().fg(Color::LightCyan)), Span::raw(": Increase the track or click volume")]),
            Line::from(vec![
                Span::styled("g", Style::default().fg(Color::LightCyan)),
                Span::raw(": start filtering for a specific song or artist."),
            ]),
            Line::from(vec![
                Span::styled("G", Style::default().fg(Color::LightCyan)),
                Span::raw(": Remove any filters and restores the entire list."),
            ]),
            Line::from("\n"),
            Line::from("When searching, hit ESC to cancel the search. Enter confirms."),
            Line::from("\n"),
            Line::from(Span::styled("Queue Commands", header_style)),
            Line::from(vec![Span::styled("+", Style::default().fg(Color::LightCyan)), Span::raw(": Adds the selected song to the queue")]),
            Line::from(vec![Span::styled("-", Style::default().fg(Color::LightCyan)), Span::raw(": Removes the selected song from the queue")]),
            Line::from(vec![Span::styled("/", Style::default().fg(Color::LightCyan)), Span::raw(": Clears the current playlist")]),
            Line::from(vec![Span::styled("*", Style::default().fg(Color::LightCyan)), Span::raw(": Randomize the current playlist")]),
            Line::from(vec![
                Span::styled("p", Style::default().fg(Color::LightCyan)),
                Span::raw(": Start playing (useful when you create a playlist)."),
            ]),
        ];

        // Create a Paragraph with the help screen content
        Paragraph::new(help_content.clone())
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Help"))
    }

    fn render_devices<'a>(&mut self, track_device: usize, click_device: usize) -> List<'a> {
        let device_ui = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Devices")
            .border_type(BorderType::Plain);

        let items: Vec<_> = crate::device::read_devices()
            .iter()
            .map(|device| {
                let selected_track = if device.position == track_device {
                    "T"
                } else {
                    "-"
                };
                let selected_click = if device.position == click_device {
                    "C"
                } else {
                    "-"
                };
                let selected_state = format!("{} {}", selected_track, selected_click);

                ListItem::new(Line::from(vec![Span::styled(
                    format!(
                        "[{}] {} : {}",
                        selected_state,
                        device.position,
                        device.name.clone()
                    ),
                    Style::default(),
                )]))
            })
            .collect();

        let list = List::new(items).block(device_ui).highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        list
    }
}
