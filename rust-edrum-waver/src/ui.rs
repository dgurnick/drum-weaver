use cpal::traits::HostTrait;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use log::{error, info};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};
use tui::backend::Backend;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Tabs,
    },
    Terminal,
};

use crate::audio::{Player, Song};
use crate::common::{get_file_paths, read_devices, Event, MenuItem, PlayerArguments};
use crate::songlist::import_songs;

pub fn run_ui(arguments: &mut PlayerArguments) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting UI");

    enable_raw_mode().expect("Can not run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    let mut selected_track_device = arguments.track_device_position;
    let mut selected_click_device = arguments.click_device_position;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Songs", "Devices", "Quit"];
    let mut active_menu_item = MenuItem::Home;

    let mut songlist_state = TableState::default();
    songlist_state.select(Some(0));

    let mut device_list_state = ListState::default();
    device_list_state.select(Some(0));

    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    let mut track_device = &available_devices[arguments.track_device_position];
    let mut click_device = &available_devices[arguments.click_device_position];

    let mut track_player = Player::new(None, track_device).expect("Could not create track player");
    let mut click_player = Player::new(None, click_device).expect("Could not create click player");
    let mut track_song: Song;
    let mut click_song: Song;

    track_player.set_playback_speed(arguments.playback_speed);
    click_player.set_playback_speed(arguments.playback_speed);

    let mut started_playing = false;
    let mut active_arguments: PlayerArguments = arguments.clone();

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
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(menu, chunks[0]);

            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Songs => {
                    let songlist_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .split(chunks[1]);
                    let song_table = render_songs(
                        &songlist_state,
                        active_arguments.clone(),
                        track_player.has_current_song(),
                    );
                    rect.render_stateful_widget(
                        song_table,
                        songlist_chunks[0],
                        &mut songlist_state,
                    );
                    //rect.render_widget(right, songlist_chunks[1]);
                }
                MenuItem::Devices => {
                    let device_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);
                    let left = render_devices(selected_track_device, selected_click_device);
                    rect.render_stateful_widget(left, device_chunks[0], &mut device_list_state);
                }
            }

            let footer_text = if track_player.has_current_song() {
                format!("Playing: {}", active_arguments.track_song.clone())
            } else {
                String::from("No song playing")
            };

            let footer_widget = Paragraph::new(footer_text);
            rect.render_widget(footer_widget, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) if event.kind == KeyEventKind::Release => match event.code {
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('s') => active_menu_item = MenuItem::Songs,
                KeyCode::Char('d') => active_menu_item = MenuItem::Devices,
                KeyCode::Char('q') => {
                    handle_q_event(&mut track_player, &mut click_player, &mut terminal)
                }
                KeyCode::Down => handle_down_event(
                    &mut active_menu_item,
                    &mut device_list_state,
                    &mut songlist_state,
                ),
                KeyCode::Up => handle_up_event(
                    &mut active_menu_item,
                    &mut device_list_state,
                    &mut songlist_state,
                ),
                KeyCode::Char(' ') => {
                    handle_space_event(&mut active_menu_item, &mut track_player, &mut click_player)
                }
                KeyCode::Char('z') => {
                    handle_z_event(&mut active_menu_item, &mut track_player, &mut click_player)
                }
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

                                track_player.set_playback_speed(arguments.playback_speed);
                                click_player.set_playback_speed(arguments.playback_speed);

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

                                started_playing = false;
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

                                track_player.set_playback_speed(arguments.playback_speed);
                                click_player.set_playback_speed(arguments.playback_speed);

                                info!("Set track device to {}", selected_track_device);
                            }
                            _ => {}
                        }
                    }
                }

                KeyCode::PageDown => {
                    handle_page_down_event(&mut active_menu_item, &mut songlist_state)
                }
                KeyCode::PageUp => handle_page_up_event(&mut active_menu_item, &mut songlist_state),
                KeyCode::Left => handle_left_arrow_event(
                    &mut active_menu_item,
                    &mut track_player,
                    &mut click_player,
                ),
                KeyCode::Right => handle_right_arrow_event(
                    &mut active_menu_item,
                    &mut track_player,
                    &mut click_player,
                ),
                KeyCode::Char('r') => {
                    handle_r_event(&mut active_menu_item, &mut track_player, &mut click_player)
                }
                KeyCode::Esc => {}

                KeyCode::Enter => match active_menu_item {
                    MenuItem::Songs => {
                        if let Some(selected) = songlist_state.selected() {
                            let (track_file, click_file) =
                                get_file_paths(&arguments.music_folder, selected + 1);

                            active_arguments = PlayerArguments {
                                music_folder: arguments.music_folder.clone(),
                                track_song: track_file,
                                click_song: click_file,
                                track_volume: arguments.track_volume,
                                click_volume: arguments.click_volume,
                                track_device_position: arguments.track_device_position,
                                click_device_position: arguments.click_device_position,
                                playback_speed: arguments.playback_speed,
                            };

                            let track_volume = Some(active_arguments.track_volume);
                            let click_volume = Some(active_arguments.click_volume);

                            let track_file = active_arguments.track_song.clone();
                            let click_file = active_arguments.click_song.clone();

                            track_song = Song::from_file(&track_file, track_volume)
                                .expect("Could not create track song");
                            click_song = Song::from_file(&click_file, click_volume)
                                .expect("Could not create click song");

                            track_player
                                .play_song_now(&track_song, None)
                                .expect("Could not play track song");
                            click_player
                                .play_song_now(&click_song, None)
                                .expect("Could not play click song");

                            started_playing = true;
                            info!("Started playing song {}", track_file.clone());
                        }
                    }
                    _ => {}
                },

                _ => {}
            },
            Event::Input(_) => {}
            Event::Tick => {
                if !track_player.has_current_song()
                    && !click_player.has_current_song()
                    && started_playing
                {
                    info!("Song ended, moving to the next song");

                    #[allow(unused_assignments)]
                    let new_position: usize;
                    if let Some(selected) = songlist_state.selected() {
                        info!("The current position is {}", selected);
                        let amount_songs = import_songs().unwrap().len();
                        if selected > amount_songs - 1 {
                            info!("Moving to position 1");
                            new_position = 0;
                        } else {
                            new_position = selected + 1;
                            info!("Moving to the next song: {}", new_position);
                        }

                        songlist_state.select(Some(new_position));

                        std::thread::sleep(std::time::Duration::from_secs(2));

                        let (track_file, click_file) =
                            get_file_paths(&active_arguments.music_folder, new_position + 1);
                        info!("The new track file is {}", track_file);

                        active_arguments = PlayerArguments {
                            music_folder: active_arguments.music_folder.clone(),
                            track_song: track_file,
                            click_song: click_file,
                            track_volume: active_arguments.track_volume,
                            click_volume: active_arguments.click_volume,
                            track_device_position: active_arguments.track_device_position,
                            click_device_position: active_arguments.click_device_position,
                            playback_speed: active_arguments.playback_speed,
                        };

                        track_song = Song::from_file(
                            &active_arguments.track_song,
                            Some(active_arguments.track_volume),
                        )
                        .expect("Could not create track song");
                        click_song = Song::from_file(
                            &active_arguments.click_song,
                            Some(active_arguments.click_volume),
                        )
                        .expect("Could not create click song");

                        std::thread::sleep(std::time::Duration::from_secs(2));
                        track_player
                            .play_song_next(&track_song, None)
                            .expect("Could not play track song");
                        click_player
                            .play_song_next(&click_song, None)
                            .expect("Could not play click song");
                    }
                }
            }
        }
    }
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Drum Karaoke",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_devices<'a>(track_device: usize, click_device: usize) -> List<'a> {
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
fn render_songs<'a>(
    songlist_state: &TableState,
    active_arguments: PlayerArguments,
    is_playing: bool,
) -> Table<'a> {
    let playlist_ui = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Songs")
        .border_type(BorderType::Plain);

    let songs = import_songs().unwrap();

    let _selected_song = songs
        .get(
            songlist_state
                .selected()
                .expect("there is always a selected song"),
        )
        .expect("exists")
        .clone();

    let mut rows = vec![];
    for song in songs {
        let mut is_selected = false;
        if is_playing {
            // TODO: This is bad
            if active_arguments.track_song.contains(song.song.as_str()) {
                is_selected = true;
            }
        }

        let selected_fg = if is_selected {
            Color::LightBlue
        } else {
            Color::White
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
                song.artist.clone(),
                Style::default().fg(selected_fg),
            )),
            Cell::from(Span::styled(
                song.song.clone(),
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
        .block(playlist_ui)
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .header(Row::new(vec![
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
            Constraint::Length(1),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]);

    song_table
}

fn handle_q_event<T>(
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
                let songs = match import_songs() {
                    Ok(songs) => songs,
                    Err(err) => {
                        error!("Could not import songs: {}", err);
                        return;
                    }
                };
                let amount_songs = songs.len();
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
        _ => {}
    }
}

fn handle_up_event(
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
                let songs = match import_songs() {
                    Ok(songs) => songs,
                    Err(err) => {
                        error!("Could not import songs: {}", err);
                        return;
                    }
                };
                let amount_songs = songs.len();
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
        _ => {}
    }
}

fn handle_space_event(
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

fn handle_page_down_event(active_menu_item: &mut MenuItem, songlist_state: &mut TableState) {
    match active_menu_item {
        MenuItem::Songs => {
            if let Some(selected) = songlist_state.selected() {
                let amount_songs = import_songs().unwrap().len();
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

fn handle_page_up_event(active_menu_item: &mut MenuItem, songlist_state: &mut TableState) {
    match active_menu_item {
        MenuItem::Songs => {
            if let Some(selected) = songlist_state.selected() {
                let amount_songs = import_songs().unwrap().len();
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
