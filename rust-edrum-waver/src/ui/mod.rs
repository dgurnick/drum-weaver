pub mod commands;
use crate::ui::commands::KeyHandler;

use cpal::traits::{DeviceTrait, HostTrait};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    terminal::enable_raw_mode,
};

use crate::device::read_devices;

use crate::common::{get_file_paths, needs_unzipping, MenuItem, PlayerArguments, UiEvent};
use crate::{
    audio::{Player, Song},
    playlist::SongRecord,
};
use log::{error, info};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use std::{env, sync::mpsc};
use std::{io, thread};

pub struct App {
    songs: Vec<SongRecord>,
    original_songs: Vec<SongRecord>,
    music_folder: Option<String>,
    track_file: Option<String>,
    click_file: Option<String>,
    track_song: Option<Song>,
    click_song: Option<Song>,
    track_volume: usize,
    click_volume: usize,
    track_device_idx: usize,
    click_device_idx: usize,
    playback_speed: f64,
    current_playlist: BTreeMap<usize, SongRecord>,
    is_quitting: bool,
    is_searching: bool,
    searching_for: String,
    is_playing_random: bool,
}

impl App {
    pub fn new(arguments: PlayerArguments, songs: Vec<SongRecord>) -> Self {
        App {
            songs: songs.clone(),
            original_songs: songs.clone(),
            music_folder: None,
            track_file: None,
            click_file: None,
            track_song: None,
            click_song: None,
            track_volume: 100,
            click_volume: 100,
            track_device_idx: arguments.track_device_position,
            click_device_idx: arguments.click_device_position,
            playback_speed: arguments.playback_speed,
            current_playlist: BTreeMap::new(),
            is_quitting: false,
            is_searching: false,
            searching_for: String::new(),
            is_playing_random: false,
        }
    }

    #[rustfmt::skip]
    pub fn run_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting UI");

        let mut has_music_folder = self.music_folder.is_some();

        enable_raw_mode().expect("Can not run in raw mode");

        let mut selected_track_device = self.track_device_idx;
        let mut selected_click_device = self.click_device_idx;

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let menu_titles = vec!["Songs", "Devices", "Help", "Quit"];
        let mut active_menu_item = MenuItem::Help;

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
        let mut footer_message = String::new();
        let mut filter_message = String::new();

        let tick_rate = Duration::from_millis(200);

        let (sender, receiver) = mpsc::channel();


        // create our transmit-receive loop
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("Polling works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        sender.send(UiEvent::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = sender.send(UiEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        loop {
            if self.is_quitting {
                terminal.draw(|f| self.confirm_exit(f))?;
            } else if !has_music_folder {
                terminal.draw(|f| self.prepare_for_folder(f))?;
            } else {
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
                            Line::from(vec![
                                Span::styled(
                                    first,
                                    Style::default()
                                        .fg(Color::LightBlue)
                                        .add_modifier(Modifier::UNDERLINED),
                                ),
                                Span::styled(rest, Style::default().fg(Color::White)),
                            ])
                        })
                        .collect();

                    let menu = Tabs::new(menu)
                        .select(active_menu_item.into())
                        .block(Block::default().title("Menu").borders(Borders::ALL))
                        .style(Style::default().fg(Color::LightBlue))
                        //.highlight_style(Style::default().fg(Color::Yellow))
                        .divider(Span::raw("|"));

                    rect.render_widget(menu, chunks[0]);

                    match active_menu_item {
                        MenuItem::Songs => {
                            let songlist_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(
                                    [Constraint::Percentage(70), Constraint::Percentage(30)]
                                        .as_ref(),
                                )
                                .split(chunks[1]);

                            let (song_table, queue_table) =
                                self.render_songs(&songlist_state, track_player.has_current_song());

                            rect.render_stateful_widget(
                                song_table,
                                songlist_chunks[0],
                                &mut songlist_state,
                            );

                            rect.render_widget(queue_table, songlist_chunks[1]);
                        }
                        MenuItem::Devices => {
                            let device_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints([Constraint::Percentage(100)].as_ref())
                                .split(chunks[1]);
                            let left =
                                self.render_devices(selected_track_device, selected_click_device);
                            rect.render_stateful_widget(
                                left,
                                device_chunks[0],
                                &mut device_list_state,
                            );
                        }
                        MenuItem::Help => {
                            rect.render_widget(self.render_help(), chunks[1]);
                        }
                    }

                    let track_device_name = match available_devices[selected_track_device].name() {
                        Ok(name) => name,
                        Err(_) => "Unknown".to_string(),
                    };

                    let click_device_name = match available_devices[selected_click_device].name() {
                        Ok(name) => name,
                        Err(_) => "Unknown".to_string(),
                    };

                    let track_volume = if let Some(_song) = &self.track_song {
                        self.track_volume
                    } else {
                        0
                    };

                    let click_volume = if let Some(_song) = &self.click_song {
                        self.click_volume
                    } else {
                        0
                    };

                    let footer_message = if self.is_searching {
                        filter_message.clone()
                    } else {
                        if track_player.has_current_song() && click_player.has_current_song() {
                            format!("Playing: {}", self.track_file.as_ref().unwrap().clone())
                        } else {
                            "No song playing".to_string()
                        }
                    };

                    let footer_device_text = format!(
                        "Track device: {} - {}% | Click device: {} - {}%",
                        track_device_name, track_volume, click_device_name, click_volume,
                    );

                    let paused_text = if track_player.is_playing() {
                        "Playing"
                    } else {
                        "Paused"
                    };

                    let footer_widget = Paragraph::new(format!(
                        "{} | {} | {}",
                        paused_text, footer_device_text, footer_message
                    ));
                    rect.render_widget(footer_widget, chunks[2]);
                })?;
            } // is quitting

            match receiver.recv()? {
                UiEvent::Input(event)
                    if event.kind == KeyEventKind::Release && ! has_music_folder =>
                {
                    match event.code {
                        KeyCode::Enter => {
                            use native_dialog::FileDialog;
                            let path = FileDialog::new().show_open_single_dir();

                            match path {
                                Ok(path) => {
                                    if let Some(p) = path {
                                        println!("The user selected this folder: {:?}", p);
                                        self.music_folder = Some(p.display().to_string());
                                        has_music_folder = true;
                                    }
                                }
                                Err(e) => {
                                    println!("The user did not select a folder: {:?}", e);
                                    std::process::exit(0);
                                }
                            };
                        }
                        KeyCode::Esc => {
                            // going to has stopped
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                UiEvent::Input(event) if event.kind == KeyEventKind::Release && self.is_searching => {
                    match event.code {
                        KeyCode::Enter => {
                            // searching has stopped
                            self.is_searching = false;
                            self.searching_for.clear();
                            filter_message.clear();
                            footer_message.clear();
                        }
                        KeyCode::Backspace =>
                        // remove last character
                        {
                            self.searching_for.pop();
                            if 1==0 && self.searching_for.is_empty() {
                                // searching has stopped
                                self.is_searching = false;
                                footer_message.clear();
                            } else {
                                self.filter_songs(
                                    self.searching_for.clone(),
                                    &mut filter_message,
                                );
                            }
                        }
                        KeyCode::Esc => {
                            // searching has stopped
                            self.is_searching = false;
                            self.searching_for.clear();
                            filter_message.clear();
                            footer_message.clear();
                        }
                        KeyCode::Char(c) => {
                            self.searching_for.push(c);

                            self.filter_songs(
                                self.searching_for.clone(),
                                &mut filter_message,
                            );
                        }
                        _ => {
                            footer_message = format!("Current search: [{}]", self.searching_for);
                        }
                    }
                }
                UiEvent::Input(event) if event.kind == KeyEventKind::Release && ! self.is_searching => {
                    
                    match event.code {
                        KeyCode::Char('1') => self.do_reduce_track_volume(&mut track_player),
                        KeyCode::Char('2') => self.do_reset_track_volume(&mut track_player),
                        KeyCode::Char('3') => self.do_increase_track_volume(&mut track_player),
                        KeyCode::Char('4') => self.do_reduce_click_volume(&mut click_player),
                        KeyCode::Char('5') => self.do_reset_click_volume(&mut click_player),
                        KeyCode::Char('6') => self.do_increase_click_volume(&mut click_player),

                        KeyCode::Char('+') => self.do_add_song_to_playlist(&mut songlist_state),
                        KeyCode::Char('-') => self.do_remove_song_from_playlist(&mut songlist_state),
                        
                        KeyCode::Char(' ') => self.do_pause_playback( &mut active_menu_item, &mut track_player, &mut click_player, ),
                        KeyCode::Char('d') => active_menu_item = MenuItem::Devices,
                        KeyCode::Char('g') => self.do_start_search(),
                        KeyCode::Char('G') => self.do_cancel_search(),
                        KeyCode::Char('h') => active_menu_item = MenuItem::Help,
                        KeyCode::Char('r') => self.do_reset_playback_speed( &mut active_menu_item, &mut track_player, &mut click_player, ),
                        KeyCode::Char('s') => active_menu_item = MenuItem::Songs,
                        KeyCode::Char('n') => self.do_check_stay_or_next(&mut track_player, &mut click_player),
                        KeyCode::Char('q') => { self.is_quitting = true; }
                        KeyCode::Char('x') => self.do_shuffle_songs(),
                        KeyCode::Char('y') => self.do_check_quit(&mut track_player, &mut click_player, &mut terminal),
                        KeyCode::Char('z') => self.do_restart_song( &mut active_menu_item, &mut track_player, &mut click_player, ),
                        
                        KeyCode::Down => self.do_select_next_item( &mut active_menu_item, &mut device_list_state, &mut songlist_state, ),
                        KeyCode::Up => self.do_select_previous_item( &mut active_menu_item, &mut device_list_state, &mut songlist_state, ),
                        KeyCode::PageDown => self.do_select_next_page(&mut active_menu_item, &mut songlist_state),
                        KeyCode::PageUp => self.do_select_previous_page(&mut active_menu_item, &mut songlist_state),

                        KeyCode::Left => self.do_reduce_playback_speed( &mut active_menu_item, &mut track_player, &mut click_player,),
                        KeyCode::Right => self.do_increase_playback_speed( &mut active_menu_item, &mut track_player, &mut click_player, ),
                        KeyCode::Home => songlist_state.select(Some(0)),
                        KeyCode::End => songlist_state.select(Some(self.songs.len() - 1)),
                        KeyCode::Delete => self.do_delete_track(&mut songlist_state, &mut track_player, &mut click_player),

                        KeyCode::Esc => {}
                        
                        
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

                                        click_device = &available_devices[selected_click_device];

                                        click_player = Player::new(None, click_device)
                                            .expect("Could not create click player");

                                        track_player.set_playback_speed(self.playback_speed);
                                        click_player.set_playback_speed(self.playback_speed);

                                        info!("Set click device to {}", selected_click_device);

                                        let beep =
                                            Song::from_file(self.get_beep_file(), None).unwrap();
                                        click_player.play_song_now(&beep, None)?;
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

                                        track_player = Player::new(None, track_device)
                                            .expect("Could not create track player");

                                        track_player.set_playback_speed(self.playback_speed);
                                        click_player.set_playback_speed(self.playback_speed);

                                        info!("Set track device to {}", selected_track_device);
                                        let beep =
                                            Song::from_file(self.get_beep_file(), None).unwrap();
                                        track_player.play_song_now(&beep, None)?;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        KeyCode::Enter => match active_menu_item {
                            MenuItem::Songs => {
                                if let Some(selected) = songlist_state.selected() {
                                    // this is wrong when we are random
                                    let selected_song = self.songs.get(selected).unwrap();

                                    if needs_unzipping(
                                        &self.music_folder.as_ref().unwrap(),
                                        &selected_song.title,
                                        &selected_song.artist,
                                        &selected_song.album,
                                    ) {
                                        footer_message = "Unzipping song".to_string();
                                    }

                                    let (track_file, click_file) = match get_file_paths(
                                        &self.music_folder.as_ref().unwrap(),
                                        &selected_song.title,
                                        &selected_song.artist,
                                        &selected_song.album,
                                    ) {
                                        Ok((track_file, click_file)) => (track_file, click_file),
                                        Err(e) => {
                                            error!("Could not get file paths: {}", e);
                                            continue;
                                        }
                                    };

                                    self.track_song = Some(
                                        Song::from_file(
                                            &track_file,
                                            Some((self.track_volume / 100) as f32),
                                        )
                                        .expect("Could not create track song"),
                                    );

                                    self.click_song = Some(
                                        Song::from_file(
                                            &click_file,
                                            Some((self.click_volume / 100) as f32),
                                        )
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
                UiEvent::Input(_) => {}
                UiEvent::Tick => {
                    if !track_player.has_current_song()
                        && !click_player.has_current_song()
                        && is_playing
                    {
                        info!("Song ended, moving to the next song");

                        footer_message = "Moving to next song in the queue".to_string();

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

                            // we need to wrap around
                            if new_position > amount_songs - 1 {
                                new_position = 0;
                            }

                            songlist_state.select(Some(new_position));
                            let selected_song = self.songs.get(new_position).unwrap();

                            if needs_unzipping(
                                &self.music_folder.as_ref().unwrap(),
                                &selected_song.title,
                                &selected_song.artist,
                                &selected_song.album,
                            ) {
                                footer_message = "Unzipping song".to_string();
                            }

                            let (track_file, click_file) = match get_file_paths(
                                &self.music_folder.as_ref().unwrap(),
                                &selected_song.title,
                                &selected_song.artist,
                                &selected_song.album,
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
                                    Some((self.track_volume / 100) as f32),
                                )
                                .expect("Could not create track song"),
                            );

                            self.click_song = Some(
                                Song::from_file(
                                    &self.click_file.clone().unwrap(),
                                    Some((self.click_volume / 100) as f32),
                                )
                                .expect("Could not create click song"),
                            );

                            track_player
                                .play_song_next(&self.track_song.as_ref().unwrap(), None)
                                .expect("Could not play track song");
                            click_player
                                .play_song_next(&self.click_song.as_ref().unwrap(), None)
                                .expect("Could not play click song");
                        }
                    } else {
                        footer_message = "".to_string();
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

        let items: Vec<_> = read_devices()
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

    fn render_songs<'a>(
        &mut self,
        songlist_state: &TableState,
        is_playing: bool,
    ) -> (Table<'a>, Table<'a>) {
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

        let queue_ui = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Queue")
            .border_type(BorderType::Plain);

        // Prepare the table data
        let rows: Vec<Row> = self
            .current_playlist
            .iter()
            .map(|(_, song)| Row::new(vec![song.artist.clone(), song.title.clone()]))
            .collect();

        let queue_table = Table::new(rows)
            .block(queue_ui)
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .header(Row::new(vec![
                Cell::from(Span::styled(
                    "Artist",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Song",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Queue")
                    .border_type(BorderType::Plain),
            )
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        (song_table, queue_table)
    }

    #[rustfmt::enable]
    fn filter_songs(
        &mut self,
        search_term: String,
        filter_message: &mut String,
    ) {
        // Filter the songs vector based on the search term
        self.songs = self.original_songs.clone();

        self.songs.retain(|song| {
            song.title.to_lowercase().contains(&search_term.clone().to_lowercase()) || song.artist.to_lowercase().contains(&search_term.clone().to_lowercase())
        });

        *filter_message = format!(
            "Search for [{}] found {} songs",
            search_term, self.songs.len());
            
        if self.songs.is_empty() {
            self.songs = self.original_songs.clone();
        }

        // if let Some(position) = self.songs.clone().iter().position(|record| {
        //     record
        //         .artist
        //         .to_lowercase()
        //         .starts_with(&search_string.clone().to_lowercase())
        // }) {
        //     // Found a matching artist at position
        //     songlist_state.select(Some(position));
        //     *footer_message = format!(
        //         "Search for {} found artist '{}'",
        //         search_string, self.songs[position].artist
        //     );
        // } else if let Some(position) = self.songs.clone().iter().position(|record| {
        //     record
        //         .title
        //         .to_lowercase()
        //         .starts_with(&search_string.clone().to_lowercase())
        // }) {
        //     songlist_state.select(Some(position));
        //     *footer_message = format!(
        //         "Search for {} found song '{}'",
        //         search_string, self.songs[position].title
        //     );
        // } else {
        //     // No match found
        //     *footer_message = format!("No song or artist starting with '{}'", search_string);
        // }
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

    fn render_help<'a>(&self) -> Paragraph<'a> {
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow);

        let help_content = vec![
            Line::from(Span::styled("General Commands", header_style)),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::LightCyan)),
                Span::raw(": Quit (boo!)."),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::LightCyan)),
                Span::raw(": Show device selection. You can pick output devices for tracks and clicks separately."),
            ]),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::LightCyan)),
                Span::raw(": Show the song list."),
            ]),
            Line::from(vec![
                Span::styled("h", Style::default().fg(Color::LightCyan)),
                Span::raw(": Show this help screen."),
            ]),
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
            Line::from(vec![
                Span::styled("r", Style::default().fg(Color::LightCyan)),
                Span::raw(": Reset the playback speed."),
            ]),
            Line::from(vec![
                Span::styled("z", Style::default().fg(Color::LightCyan)),
                Span::raw(": Restart the song that is playing."),
            ]),
            Line::from(vec![
                Span::styled("SPACE", Style::default().fg(Color::LightCyan)),
                Span::raw(": Pause or continue the song that is playing"),
            ]),
            Line::from(vec![
                Span::styled("HOME or END", Style::default().fg(Color::LightCyan)),
                Span::raw(": Move to the first or last song in the queue or song list."),
            ]),
            Line::from(vec![
                Span::styled("x", Style::default().fg(Color::LightCyan)),
                Span::raw(": Shuffle or un-shuffle the playlist"),
            ]),
            Line::from(vec![
                Span::styled("1 or 4", Style::default().fg(Color::LightCyan)),
                Span::raw(": Lower the track or click volume"),
            ]),
            Line::from(vec![
                Span::styled("2 or 5", Style::default().fg(Color::LightCyan)),
                Span::raw(": Reset the track or click volume"),
            ]),
            Line::from(vec![
                Span::styled("3 or 6", Style::default().fg(Color::LightCyan)),
                Span::raw(": Increase the track or click volume"),
            ]),
            Line::from(vec![
                Span::styled("g", Style::default().fg(Color::LightCyan)),
                Span::raw(": start searching for a specific song or artist."),
            ]),
            Line::from("\n"),
            Line::from("When searching, hit ESC to cancel the search. Enter confirms."),
            Line::from("\n"),
            Line::from(Span::styled("Queue Commands", header_style)),
            Line::from(vec![
                Span::styled("+", Style::default().fg(Color::LightCyan)),
                Span::raw(": Adds the selected song to the queue"),
            ]),
            Line::from(vec![
                Span::styled("-", Style::default().fg(Color::LightCyan)),
                Span::raw(": Removes the selected song from the queue"),
            ]),
        ];

        // Create a Paragraph with the help screen content
        Paragraph::new(help_content.clone())
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Help"))
    }

    fn confirm_exit<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(size);

        let paragraph = Paragraph::new(Span::styled(
            "Hi there",
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(paragraph, chunks[0]);

        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::Black));
        f.render_widget(block, chunks[1]);

        let block = Block::default()
            .title("Are you sure you want to leave? (Y/N)")
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center);

        let area = self.centered_rect(60, 20, size);
        //f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
    }

    fn prepare_for_folder<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(size);

        let paragraph = Paragraph::new(Span::styled(
            "Hi there. You need to select a folder where your music collection is stored. This is typically something called 'Drumless'. Click ENTER to choose your music folder. ESC to close.",
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(paragraph, chunks[0]);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    fn get_beep_file(&self) -> String {
        let mut path = env::current_dir().expect("Failed to get current exe path");

        // Append the relative path to your asset
        path.push("assets");
        path.push("beep.wav");

        path.display().to_string()
    }

}
