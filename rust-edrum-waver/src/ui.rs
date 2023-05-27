use cpal::traits::HostTrait;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::common::{get_file_paths, read_devices, Event, MenuItem, PlayerArguments};
use crate::{
    audio::{Player, Song},
    playlist::SongRecord,
};
use log::{error, info};
use std::sync::mpsc;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use std::{io, thread};
use tui::backend::Backend;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Tabs,
    },
    Terminal,
};

pub struct App {
    songs: Vec<SongRecord>,
    music_folder: Option<String>,
    track_file: Option<String>,
    click_file: Option<String>,
    track_song: Option<Song>,
    click_song: Option<Song>,
    track_volume: Option<f32>,
    click_volume: Option<f32>,
    track_device_idx: usize,
    click_device_idx: usize,
    playback_speed: f64,
    current_playlist: BTreeMap<usize, SongRecord>,
}

impl App {
    pub fn new(arguments: PlayerArguments, songs: Vec<SongRecord>) -> App {
        App {
            songs: songs,
            music_folder: None,
            track_file: None,
            click_file: None,
            track_song: None,
            click_song: None,
            track_volume: Some(arguments.track_volume),
            click_volume: Some(arguments.click_volume),
            track_device_idx: arguments.track_device_position,
            click_device_idx: arguments.click_device_position,
            playback_speed: arguments.playback_speed,
            current_playlist: BTreeMap::new(),
        }
    }

    pub fn run_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting UI");

        if self.music_folder.is_none() {
            // let choice = dialog::FileSelection::new("Please select a file")
            //     .title("File Selection")
            //     //.path("/home/user/Downloads")
            //     .show()
            //     .expect("Could not display dialog box");
            // arguments.music_folder = Some(choice.unwrap());
            use native_dialog::FileDialog;
            let path = FileDialog::new().show_open_single_dir();

            match path {
                Ok(path) => {
                    if let Some(p) = path {
                        println!("The user selected this folder: {:?}", p);
                        self.music_folder = Some(p.display().to_string());
                    }
                }
                Err(e) => {
                    println!("The user did not select a folder: {:?}", e);
                }
            };
        }

        enable_raw_mode().expect("Can not run in raw mode");

        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);

        let mut selected_track_device = self.track_device_idx;
        let mut selected_click_device = self.click_device_idx;

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let menu_titles = vec!["Songs", "Devices", "Quit"];
        let mut active_menu_item = MenuItem::Songs;

        let mut songlist_state = TableState::default();
        songlist_state.select(Some(0));

        let mut device_list_state = ListState::default();
        device_list_state.select(Some(0));

        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

        let mut track_device = &available_devices[self.track_device_idx];
        let mut click_device = &available_devices[self.click_device_idx];

        let mut track_player =
            Player::new(None, track_device).expect("Could not create track player");
        let mut click_player =
            Player::new(None, click_device).expect("Could not create click player");

        track_player.set_playback_speed(self.playback_speed);
        click_player.set_playback_speed(self.playback_speed);

        let mut is_playing = false;
        let mut is_going_to = false;
        let mut going_to = String::new();
        let mut footer_message = String::new();

        // create our transmit-receive loop
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("Polling works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        loop {
            terminal.draw(|rect| {
                let size = rect.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(2),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                let menu = menu_titles
                    .iter()
                    .map(|t| {
                        let (first, rest) = t.split_at(1);
                        Spans::from(vec![
                            Span::styled(
                                first,
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::UNDERLINED),
                            ),
                            Span::styled(rest, Style::default().fg(Color::White)),
                        ])
                    })
                    .collect();

                let menu = Tabs::new(menu)
                    .select(active_menu_item.into())
                    .block(Block::default().title("Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    //.highlight_style(Style::default().fg(Color::Yellow))
                    .divider(Span::raw("|"));

                rect.render_widget(menu, chunks[0]);

                match active_menu_item {
                    MenuItem::Songs => {
                        let songlist_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(100)].as_ref())
                            .split(chunks[1]);
                        let song_table =
                            self.render_songs(&songlist_state, track_player.has_current_song());

                        rect.render_stateful_widget(
                            song_table,
                            songlist_chunks[0],
                            &mut songlist_state,
                        );
                    }
                    MenuItem::Devices => {
                        let device_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                            )
                            .split(chunks[1]);
                        let left =
                            self.render_devices(selected_track_device, selected_click_device);
                        rect.render_stateful_widget(left, device_chunks[0], &mut device_list_state);
                    }
                }

                let footer_text = if footer_message.is_empty() {
                    if track_player.has_current_song() {
                        format!("Playing: {}", self.track_file.as_ref().unwrap().clone())
                    } else {
                        "No song playing".to_string()
                    }
                } else {
                    footer_message.clone()
                };

                let footer_widget = Paragraph::new(footer_text);
                rect.render_widget(footer_widget, chunks[2]);
            })?;

            match rx.recv()? {
                Event::Input(event) if event.kind == KeyEventKind::Release && is_going_to => {
                    match event.code {
                        KeyCode::Enter => {
                            // going to has stopped
                            is_going_to = false;
                            going_to.clear();
                            footer_message.clear();
                        }
                        KeyCode::Backspace =>
                        // remove last character
                        {
                            going_to.pop();
                            if going_to.is_empty() {
                                // going to has stopped
                                is_going_to = false;
                                footer_message.clear();
                            } else {
                                self.find_song(
                                    &mut songlist_state,
                                    going_to.clone(),
                                    &mut footer_message,
                                );
                            }
                        }
                        KeyCode::Esc => {
                            // going to has stopped
                            is_going_to = false;
                            going_to.clear();
                            footer_message.clear();
                        }
                        KeyCode::Char(c) => {
                            going_to.push(c);

                            self.find_song(
                                &mut songlist_state,
                                going_to.clone(),
                                &mut footer_message,
                            );
                        }
                        _ => {
                            footer_message = format!("Current search: [{}]", going_to);
                        }
                    }
                }
                Event::Input(event) if event.kind == KeyEventKind::Release && !is_going_to => {
                    match event.code {
                        KeyCode::Char('+') => {
                            if let Some(selected) = songlist_state.selected() {
                                // add it to the queue. We can keep addint. No issue.
                                let song = self.songs[selected].clone();
                                let position = self
                                    .current_playlist
                                    .values()
                                    .position(|song_record| song_record.title == song.title);

                                if position.is_none() {
                                    self.current_playlist
                                        .insert(self.current_playlist.len() + 1, song.clone());
                                    info!("Added song to queue: {}", &song.title);
                                }

                                for (i, song) in self.current_playlist.values().enumerate() {
                                    info!("{}: {}", i, song.title);
                                }

                                self.reindex_playlist();
                            }
                        }
                        KeyCode::Char('-') => {
                            if let Some(selected) = songlist_state.selected() {
                                // add it to the queue. We can keep addint. No issue.
                                let song = self.songs[selected].clone();
                                let position = self
                                    .current_playlist
                                    .values()
                                    .position(|song_record| song_record.title == song.title);

                                if let Some(pos) = position {
                                    self.current_playlist.remove(&(&pos + 1));
                                    info!("Removed song from queue: {}", song.title);
                                }

                                for (i, song) in self.current_playlist.values().enumerate() {
                                    info!("{}: {}", i, song.title);
                                }

                                self.reindex_playlist();
                            }
                        }
                        KeyCode::Char('s') => active_menu_item = MenuItem::Songs,
                        KeyCode::Char('d') => active_menu_item = MenuItem::Devices,
                        KeyCode::Char('q') => {
                            self.handle_q_event(&mut track_player, &mut click_player, &mut terminal)
                        }
                        KeyCode::Char('g') => {
                            going_to = String::new();
                            is_going_to = true;
                        }
                        KeyCode::Down => self.handle_down_event(
                            &mut active_menu_item,
                            &mut device_list_state,
                            &mut songlist_state,
                        ),
                        KeyCode::Up => self.handle_up_event(
                            &mut active_menu_item,
                            &mut device_list_state,
                            &mut songlist_state,
                        ),
                        KeyCode::Char(' ') => self.handle_space_event(
                            &mut active_menu_item,
                            &mut track_player,
                            &mut click_player,
                        ),
                        KeyCode::Char('z') => self.handle_z_event(
                            &mut active_menu_item,
                            &mut track_player,
                            &mut click_player,
                        ),
                        KeyCode::Char('c') => {
                            // I won't refactor this into another function because it uses everything and I'm dumb
                            if event.kind == KeyEventKind::Release {
                                match active_menu_item {
                                    MenuItem::Devices => {
                                        if let Some(selected) = device_list_state.selected() {
                                            selected_click_device = selected;
                                        }

                                        track_player.force_remove_next_song()?;
                                        click_player.force_remove_next_song()?;
                                        track_player.stop();
                                        click_player.stop();

                                        track_device = &available_devices[selected_track_device];
                                        click_device = &available_devices[selected_click_device];

                                        track_player = Player::new(None, track_device)
                                            .expect("Could not create track player");
                                        click_player = Player::new(None, click_device)
                                            .expect("Could not create click player");

                                        track_player.set_playback_speed(self.playback_speed);
                                        click_player.set_playback_speed(self.playback_speed);

                                        info!("Set click device to {}", selected_click_device);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        KeyCode::Char('t') => {
                            // I won't refactor this into another function because it uses everything and I'm dumb
                            if event.kind == KeyEventKind::Release {
                                match active_menu_item {
                                    MenuItem::Devices => {
                                        if let Some(selected) = device_list_state.selected() {
                                            selected_track_device = selected;
                                        }

                                        is_playing = false;
                                        track_player.force_remove_next_song()?;
                                        click_player.force_remove_next_song()?;
                                        track_player.stop();
                                        click_player.stop();

                                        track_device = &available_devices[selected_track_device];
                                        click_device = &available_devices[selected_click_device];

                                        track_player = Player::new(None, track_device)
                                            .expect("Could not create track player");
                                        click_player = Player::new(None, click_device)
                                            .expect("Could not create click player");

                                        track_player.set_playback_speed(self.playback_speed);
                                        click_player.set_playback_speed(self.playback_speed);

                                        info!("Set track device to {}", selected_track_device);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        KeyCode::PageDown => {
                            self.handle_page_down_event(&mut active_menu_item, &mut songlist_state)
                        }
                        KeyCode::PageUp => {
                            self.handle_page_up_event(&mut active_menu_item, &mut songlist_state)
                        }
                        KeyCode::Left => self.handle_left_arrow_event(
                            &mut active_menu_item,
                            &mut track_player,
                            &mut click_player,
                        ),
                        KeyCode::Right => self.handle_right_arrow_event(
                            &mut active_menu_item,
                            &mut track_player,
                            &mut click_player,
                        ),
                        KeyCode::Char('r') => self.handle_r_event(
                            &mut active_menu_item,
                            &mut track_player,
                            &mut click_player,
                        ),
                        KeyCode::Esc => {}

                        KeyCode::Enter => match active_menu_item {
                            MenuItem::Songs => {
                                if let Some(selected) = songlist_state.selected() {
                                    let (track_file, click_file) = match get_file_paths(
                                        &self.music_folder.as_ref().unwrap(),
                                        selected + 1,
                                    ) {
                                        Ok((track_file, click_file)) => (track_file, click_file),
                                        Err(e) => {
                                            error!("Could not get file paths: {}", e);
                                            continue;
                                        }
                                    };

                                    self.track_song = Some(
                                        Song::from_file(&track_file, self.track_volume)
                                            .expect("Could not create track song"),
                                    );

                                    self.click_song = Some(
                                        Song::from_file(&click_file, self.click_volume.clone())
                                            .expect("Could not create click song"),
                                    );

                                    track_player
                                        .play_song_now(&self.track_song.as_ref().unwrap(), None)
                                        .expect("Could not play track song");
                                    click_player
                                        .play_song_now(&self.click_song.as_ref().unwrap(), None)
                                        .expect("Could not play click song");

                                    is_playing = true;
                                    self.track_file = Some(track_file);
                                    self.click_file = Some(click_file);
                                }
                            }
                            _ => {}
                        },

                        _ => {}
                    }
                }
                Event::Input(_) => {}
                Event::Tick => {
                    if !track_player.has_current_song()
                        && !click_player.has_current_song()
                        && is_playing
                    {
                        info!("Song ended, moving to the next song");

                        #[allow(unused_assignments)]
                        let mut new_position: usize;

                        // move to first in playlist if it's there
                        if !self.current_playlist.is_empty() {
                            let (_, song_record) = self.current_playlist.pop_first().unwrap();

                            // find the position of the song_title in our song list
                            if let Some(index) = self
                                .songs
                                .iter()
                                .position(|song| song.title == song_record.title)
                            {
                                // THIS IS A TERRIBLE HACK. I'm sorry.
                                new_position = index;
                                songlist_state.select(Some(new_position));
                            }

                            self.reindex_playlist();
                        }

                        if let Some(selected) = songlist_state.selected() {
                            info!("The current position is {}", selected);
                            let amount_songs = self.songs.len();
                            if selected > amount_songs - 1 {
                                info!("Moving to position 1");
                                new_position = 0;
                            } else {
                                new_position = selected + 1;
                                info!("Moving to the next song: {}", new_position);
                            }

                            songlist_state.select(Some(new_position));

                            std::thread::sleep(std::time::Duration::from_secs(2));

                            let (track_file, click_file) = match get_file_paths(
                                &self.music_folder.as_ref().unwrap(),
                                new_position + 1,
                            ) {
                                Ok((track_file, click_file)) => (track_file, click_file),
                                Err(e) => {
                                    error!("Could not get file paths: {}", e);
                                    continue;
                                }
                            };

                            info!("The new track file is {}", track_file);

                            self.track_file = Some(track_file);
                            self.click_file = Some(click_file);

                            self.track_song = Some(
                                Song::from_file(
                                    &self.track_file.clone().unwrap(),
                                    self.track_volume,
                                )
                                .expect("Could not create track song"),
                            );

                            self.click_song = Some(
                                Song::from_file(
                                    &self.click_file.clone().unwrap(),
                                    self.click_volume,
                                )
                                .expect("Could not create click song"),
                            );

                            std::thread::sleep(std::time::Duration::from_secs(2));
                            track_player
                                .play_song_next(&self.track_song.as_ref().unwrap(), None)
                                .expect("Could not play track song");
                            click_player
                                .play_song_next(&self.click_song.as_ref().unwrap(), None)
                                .expect("Could not play click song");
                        }
                    }
                }
            }
        }
    }

    fn render_devices<'a>(&mut self, track_device: usize, click_device: usize) -> List<'a> {
        let device_ui = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Devices")
            .border_type(BorderType::Plain);

        let devices = read_devices().expect("can't fetch device list");
        let items: Vec<_> = devices
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

                ListItem::new(Spans::from(vec![Span::styled(
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

    // TODO: Add * if song is in the current playlist
    fn render_songs<'a>(&mut self, songlist_state: &TableState, is_playing: bool) -> Table<'a> {
        let songlist_ui = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Songs")
            .border_type(BorderType::Plain);

        let _selected_song = self
            .songs
            .get(
                songlist_state
                    .selected()
                    .expect("there is always a selected song"),
            )
            .expect("exists")
            .clone();

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
                Color::White
            };

            let playlist_position = self
                .current_playlist
                .values()
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
            ]);
            rows.push(row);
        }

        let song_table = Table::new(rows)
            .block(songlist_ui)
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .header(Row::new(vec![
                Cell::from(Span::raw(" ")),
                Cell::from(Span::raw(" ")),
                Cell::from(Span::styled(
                    "Artist",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Song",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Album",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Genre",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Detail")
                    .border_type(BorderType::Plain),
            )
            .widths(&[
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]);

        // let playlist_ui = Block::default()
        //     .borders(Borders::ALL)
        //     .style(Style::default().fg(Color::White))
        //     .title("Queue")
        //     .border_type(BorderType::Plain);

        // let mut rows = vec![];
        // for song in self.current_playlist.values() {
        //     let row = Row::new(vec![
        //         Cell::from(Span::styled(
        //             song.artist.clone(),
        //             Style::default().fg(Color::LightGreen),
        //         )),
        //         Cell::from(Span::styled(
        //             song.title.clone(),
        //             Style::default().fg(Color::LightGreen),
        //         )),
        //     ]);
        //     rows.push(row);
        // }

        // let playlist_table = Table::new(rows)
        //     .block(playlist_ui)
        //     .highlight_style(
        //         Style::default()
        //             .bg(Color::Yellow)
        //             .fg(Color::Black)
        //             .add_modifier(Modifier::BOLD),
        //     )
        //     .header(Row::new(vec![
        //         Cell::from(Span::styled(
        //             "Artist",
        //             Style::default().add_modifier(Modifier::BOLD),
        //         )),
        //         Cell::from(Span::styled(
        //             "Song",
        //             Style::default().add_modifier(Modifier::BOLD),
        //         )),
        //     ]))
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .style(Style::default().fg(Color::White))
        //             .title("Detail")
        //             .border_type(BorderType::Plain),
        //     )
        //     .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        // (song_table, playlist_table)
        song_table
    }

    fn handle_q_event<T>(
        &mut self,
        track_player: &mut Player,
        click_player: &mut Player,
        terminal: &mut Terminal<T>,
    ) where
        T: Backend,
    {
        info!("Quitting");
        track_player.stop();
        click_player.stop();
        disable_raw_mode().expect("Can not disable raw mode");
        terminal.clear().expect("Failed to clear the terminal");
        terminal.show_cursor().expect("Failed to show cursor");
        std::process::exit(0);
    }

    fn handle_down_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Devices => {
                if let Some(selected) = device_list_state.selected() {
                    let amount_devices = read_devices().expect("can't fetch device list").len();
                    if selected >= amount_devices - 1 {
                        device_list_state.select(Some(0));
                    } else {
                        device_list_state.select(Some(selected + 1));
                    }
                }
                info!("Set device to {}", device_list_state.selected().unwrap());
            }
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    #[allow(unused_assignments)]
                    let mut new_position = selected;
                    if selected >= amount_songs - 1 {
                        new_position = 0;
                    } else {
                        new_position = selected + 1;
                    }
                    songlist_state.select(Some(new_position));
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
        }
    }

    fn handle_up_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Devices => {
                if let Some(selected) = device_list_state.selected() {
                    let amount_devices = read_devices().expect("can't fetch device list").len();
                    if selected > 0 {
                        device_list_state.select(Some(selected - 1));
                    } else {
                        device_list_state.select(Some(amount_devices - 1));
                    }
                }
                info!("Set device to {}", device_list_state.selected().unwrap());
            }
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    #[allow(unused_assignments)]
                    let mut new_position = 0;
                    if selected > 0 {
                        new_position = selected - 1;
                    } else {
                        new_position = amount_songs - 1;
                    }
                    songlist_state.select(Some(new_position));
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
        }
    }

    fn handle_space_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                track_player.set_playing(!track_player.is_playing());
                click_player.set_playing(!click_player.is_playing());

                info!("Stopped playback of song");
            }
            _ => {}
        }
    }

    fn handle_z_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if track_player.is_playing() {
                    track_player.seek(Duration::from_secs(0));
                    click_player.seek(Duration::from_secs(0));
                }
                info!("Restarted song");
            }
            _ => {}
        }
    }

    fn handle_page_down_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected + 10 > amount_songs {
                        songlist_state.select(Some(0));
                    } else {
                        songlist_state.select(Some(selected + 10));
                    }
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn handle_page_up_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected > 10 {
                        songlist_state.select(Some(selected - 10));
                    } else {
                        songlist_state.select(Some(amount_songs - 1));
                    }
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn handle_left_arrow_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                let current_speed = track_player.get_playback_speed();
                if current_speed > 0.1 {
                    track_player.set_playback_speed(current_speed - 0.01);
                    click_player.set_playback_speed(current_speed - 0.01);
                }

                info!(
                    "Set playback speed to {}",
                    track_player.get_playback_speed()
                );
            }
            _ => {}
        }
    }

    fn handle_right_arrow_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                let current_speed = track_player.get_playback_speed();
                if current_speed > 0.1 {
                    track_player.set_playback_speed(current_speed + 0.01);
                    click_player.set_playback_speed(current_speed + 0.01);
                }

                info!(
                    "Set playback speed to {}",
                    track_player.get_playback_speed()
                );
            }
            _ => {}
        }
    }

    fn handle_r_event(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                track_player.set_playback_speed(1.0);
                click_player.set_playback_speed(1.0);
                info!("Reset playback speed to 1x ");
            }
            _ => {}
        }
    }

    fn find_song(
        &self,
        songlist_state: &mut TableState,
        going_to: String,
        footer_message: &mut String,
    ) {
        if let Some(position) = self.songs.clone().iter().position(|record| {
            record
                .artist
                .to_lowercase()
                .starts_with(&going_to.clone().to_lowercase())
        }) {
            // Found a matching artist at position
            songlist_state.select(Some(position));
            *footer_message = format!(
                "Search for {} found artist '{}'",
                going_to, self.songs[position].artist
            );
        } else if let Some(position) = self.songs.clone().iter().position(|record| {
            record
                .title
                .to_lowercase()
                .starts_with(&going_to.clone().to_lowercase())
        }) {
            songlist_state.select(Some(position));
            *footer_message = format!(
                "Search for {} found song '{}'",
                going_to, self.songs[position].title
            );
        } else {
            // No match found
            *footer_message = format!("No song or artist starting with '{}'", going_to);
        }
    }

    fn reindex_playlist(&mut self) {
        let mut idx = 0;

        let values: Vec<SongRecord> = self.current_playlist.values().cloned().collect();
        self.current_playlist.clear();

        for song_record in values {
            idx += 1;
            self.current_playlist.insert(idx, song_record);
        }
    }
}
