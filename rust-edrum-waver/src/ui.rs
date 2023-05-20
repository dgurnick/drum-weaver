use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};
use crossterm::event::KeyEvent;
use crossterm::terminal;
use log::{debug, error, info, trace, warn};
use log4rs;

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use tui::backend::Backend;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
    Terminal,
};

//use lazy_static::lazy_static;

use crate::common::{Event, PlayerArguments, get_file_paths, play_song};
use crate::common::MenuItem;
use crate::common::read_playlists;
use crate::common::read_songs;
use crate::common::read_devices;
use crate::common::SongRecord;
use crate::lib::{Player, Song};
use cpal::Device;
use cpal::traits::HostTrait;

pub fn run_ui(arguments: PlayerArguments) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting UI");

    enable_raw_mode().expect("Can not run in raw mode");
    
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    let mut selected_track_device = arguments.track_device_position;
    let mut selected_click_device = arguments.click_device_position;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear();

    let menu_titles = vec!["Home", "Playlists", "Songs", "Devices", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut playlist_state = ListState::default();
    playlist_state.select(Some(0));

    let mut songlist_state = ListState::default();
    songlist_state.select(Some(0));

    let mut device_list_state = ListState::default();
    device_list_state.select(Some(0));

    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    let mut track_device = &available_devices[arguments.track_device_position];
    let mut click_device = &available_devices[arguments.click_device_position];

    let mut track_player = Player::new(None, track_device).expect("Could not create track player");
    let mut click_player = Player::new(None, click_device).expect("Could not create click player");

    track_player.set_playback_speed(arguments.playback_speed);
    click_player.set_playback_speed(arguments.playback_speed);

    let mut started_playing = false;
    let mut active_arguments: PlayerArguments = arguments.clone();

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

            let copyright = Paragraph::new("Drum karaoke player")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::all())
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),

                );

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

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Playlists => {
                    let playlist_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_playlists(&playlist_state);
                    rect.render_stateful_widget(left, playlist_chunks[0], &mut playlist_state);
                    rect.render_widget(right, playlist_chunks[1]);
                },
                MenuItem::Songs => {
                    let songlist_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_songs(&songlist_state);
                    rect.render_stateful_widget(left, songlist_chunks[0], &mut songlist_state);
                    rect.render_widget(right, songlist_chunks[1]);
                },
                MenuItem::Devices => {
                    let device_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);
                    let left = render_devices(selected_track_device, selected_click_device);
                    rect.render_stateful_widget(left, device_chunks[0], &mut device_list_state);
                },

            }
            rect.render_widget(copyright, chunks[2]);

        })?;

        match rx.recv()? {

            Event::Input(event) => match event.code {
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Playlists,
                KeyCode::Char('s') => active_menu_item = MenuItem::Songs,
                KeyCode::Char('d') => active_menu_item = MenuItem::Devices,
                KeyCode::Char('q') => handle_q_event(event, &mut track_player, &mut click_player, &mut terminal),
                KeyCode::Down => {
                    
                    if event.kind == KeyEventKind::Release {

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

                            },
                            MenuItem::Playlists => {
                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected >= amount_playlists - 1 {
                                        playlist_state.select(Some(0));
                                    } else {
                                        playlist_state.select(Some(selected + 1));
                                    }
                                }
                                info!("Set playlist to {}", playlist_state.selected().unwrap());

                            },
                            MenuItem::Songs => {
                                if let Some(selected) = songlist_state.selected() {
                                    let amount_songs = read_songs().expect("can't fetch play list").len();
                                    if selected >= amount_songs - 1 {
                                        songlist_state.select(Some(0));
                                    } else {
                                        songlist_state.select(Some(selected + 1));
                                    }
                                }
                                info!("Set song to {}", songlist_state.selected().unwrap());

                            },
                            _ => {}

                        }

                    }
                }
                KeyCode::Up => {
                    if event.kind == KeyEventKind::Release {

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
                            },
                            MenuItem::Playlists => {

                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected > 0 {
                                        playlist_state.select(Some(selected - 1));
                                    } else {
                                        playlist_state.select(Some(amount_playlists - 1));
                                    }
                                }
                                info!("Set playlist to {}", playlist_state.selected().unwrap());
                            },
                            MenuItem::Songs => {

                                if let Some(selected) = songlist_state.selected() {
                                    let amount_songs = read_songs().expect("can't fetch songs").len();
                                    if selected > 0 {
                                        songlist_state.select(Some(selected - 1));
                                    } else {
                                        songlist_state.select(Some(amount_songs - 1));
                                    }
                                }
                                info!("Set song to {}", songlist_state.selected().unwrap());

                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::Char(' ') => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Songs => {
                                track_player.set_playing(! track_player.is_playing());
                                click_player.set_playing(! click_player.is_playing());

                                info!("Stopped playback of song");

                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::Char('z') => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Songs => {
                                if track_player.is_playing() {
                                    track_player.seek(Duration::from_secs(0));
                                    click_player.seek(Duration::from_secs(0));
                                }
                                info!("Restarted song");

                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::Char('c') => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Devices => {
                                if let Some(selected) = device_list_state.selected() {
                                    selected_click_device = selected;
                                }

                                track_player.stop();
                                click_player.stop();

                                track_device = &available_devices[selected_track_device];
                                click_device = &available_devices[selected_click_device];

                                track_player = Player::new(None, track_device).expect("Could not create track player");
                                click_player = Player::new(None, click_device).expect("Could not create click player");

                                track_player.set_playback_speed(arguments.playback_speed);
                                click_player.set_playback_speed(arguments.playback_speed);

                                info!("Set click device to {}", selected_click_device);
                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::Char('t') => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Devices => {
                                if let Some(selected) = device_list_state.selected() {
                                    selected_track_device = selected;
                                }

                                track_player.stop();
                                click_player.stop();

                                track_device = &available_devices[selected_track_device];
                                click_device = &available_devices[selected_click_device];

                                track_player = Player::new(None, track_device).expect("Could not create track player");
                                click_player = Player::new(None, click_device).expect("Could not create click player");

                                track_player.set_playback_speed(arguments.playback_speed);
                                click_player.set_playback_speed(arguments.playback_speed);

                                info!("Set track device to {}", selected_track_device);
                                
                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::PageDown => {
                    
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {
                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected >= amount_playlists - 1 {
                                        playlist_state.select(Some(0));
                                    } else {
                                        playlist_state.select(Some(selected + 10));
                                    }
                                }

                                info!("Set playlist to {}", playlist_state.selected().unwrap());

                            },
                            MenuItem::Songs => {
                                if let Some(selected) = songlist_state.selected() {
                                    let amount_songs = read_songs().expect("can't fetch play list").len();
                                    if selected + 10 > amount_songs {
                                        songlist_state.select(Some(0));
                                    } else {
                                        songlist_state.select(Some(selected + 10));
                                    }
                                }
                                info!("Set song to {}", songlist_state.selected().unwrap());

                            },
                            _ => {}

                        }

                    }
                }
                KeyCode::PageUp => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {

                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected > 0 {
                                        playlist_state.select(Some(selected - 10));
                                    } else {
                                        playlist_state.select(Some(amount_playlists - 1));
                                    }
                                }
                                info!("Set playlist to {}",  playlist_state.selected().unwrap());
                            },
                            MenuItem::Songs => {

                                if let Some(selected) = songlist_state.selected() {
                                    let amount_songs = read_songs().expect("can't fetch songs").len();
                                    if selected > 10 {
                                        songlist_state.select(Some(selected - 10));
                                    } else {
                                        songlist_state.select(Some(amount_songs - 1));
                                    }
                                }
                                info!("Set song to {}", songlist_state.selected().unwrap());

                            },
                            _ => {}

                        }

                    }
                },

                KeyCode::Left => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Songs => {
                                let current_speed = track_player.get_playback_speed();
                                if current_speed > 0.1 {
                                    track_player.set_playback_speed(current_speed - 0.01);
                                    click_player.set_playback_speed(current_speed - 0.01);
                                }

                                info!("Set playback speed to {}", track_player.get_playback_speed());
                            },
                            _ => {}

                        }
                    }
                },

                KeyCode::Right => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Songs => {
                                let current_speed = track_player.get_playback_speed();
                                track_player.set_playback_speed(current_speed + 0.01);
                                click_player.set_playback_speed(current_speed + 0.01);

                                info!("Set playback speed to {}", track_player.get_playback_speed());

                            },
                            _ => {}

                        }
                    }
                },

                KeyCode::Char('r') => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Songs => {
                                track_player.set_playback_speed(1.0);
                                click_player.set_playback_speed(1.0);
                                info!("Reset playback speed to 1x ");
                            },
                            _ => {}

                        }

                    }
                },
                        
                KeyCode::Enter => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {

                                info!("TODO Play selected play list");

                            },
                            MenuItem::Songs => {

                                if let Some(selected) = songlist_state.selected() {     

                                    let (track_file, click_file) = get_file_paths(&arguments.music_folder, selected + 1);

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

                                    let track_song = Song::from_file(&track_file, track_volume).expect("Could not create track song");
                                    let click_song = Song::from_file(&click_file, click_volume).expect("Could not create click song");
                                
                                    track_player.play_song_now(&track_song, None).expect("Could not play track song");
                                    click_player.play_song_now(&click_song, None).expect("Could not play click song");

                                    started_playing = true;
                                    info!("Started playing song {}", track_file.clone());

                                }

                            },
                            _ => {}

                        }

                    }
                },
                _ => {}
            },
            Event::Tick => {}

        }

        if !track_player.has_current_song()
           && !click_player.has_current_song()
           && started_playing
        {
            active_arguments = match handle_song_end_conditions(
                &mut track_player,
                &mut click_player,
                &mut songlist_state,
                &active_arguments,
            ) {
                Ok(arguments) => arguments,
                Err(err) => {
                    error!("Could not handle song end conditions: {}", err);
                    break;
                }
            };
        }
  
    }


    Ok(())
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
        Spans::from(vec![Span::raw("Press 'p' to access playlists, 'a' to add a new playlist and 'd' to delete the currently selected playlist.")]),
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

fn render_playlists<'a>(playlist_state: &ListState) -> (List<'a>, Table<'a>) {
    let playlist_ui = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Playlists")
        .border_type(BorderType::Plain);

    let playlists = read_playlists().expect("can fetch play list");
    let items: Vec<_> = playlists
        .iter()
        .map(|playlist| {
            ListItem::new(Spans::from(vec![Span::styled(
                playlist.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_playlist = playlists
        .get(
            playlist_state
                .selected()
                .expect("there is always a selected playlist"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(playlist_ui).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let playlist_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_playlist.id.to_string())),
        Cell::from(Span::raw(selected_playlist.name)),
        Cell::from(Span::raw(selected_playlist.category)),
        Cell::from(Span::raw(selected_playlist.age.to_string())),
        Cell::from(Span::raw(selected_playlist.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
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
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, playlist_detail)
}       

fn render_devices<'a>(track_device: usize, click_device: usize) -> (List<'a>) {
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
            let mut selected_state = format!("{} {}", selected_track, selected_click);

            ListItem::new(Spans::from(vec![Span::styled(
                format!("[{}] {} : {}", selected_state, device.position, device.name.clone()),
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

fn render_songs<'a>(songlist_state: &ListState) -> (List<'a>, Table<'a>) {

    let playlist_ui = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Songs")
        .border_type(BorderType::Plain);

    let songs = read_songs().expect("can fetch song list");
    let items: Vec<_> = songs
        .iter()
        .map(|song| {
            ListItem::new(Spans::from(vec![Span::styled(
                format!("{} - {}", song.artist.clone(), song.song.clone()),
                Style::default(),
            )]))
        })
        .collect();

    let selected_song = songs
        .get(
            songlist_state
                .selected()
                .expect("there is always a selected song"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(playlist_ui).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let song_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_song.song)),
        Cell::from(Span::raw(selected_song.genre)),
        Cell::from(Span::raw(selected_song.artist)),
        Cell::from(Span::raw(selected_song.album)),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Song",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Genre",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Artist",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Album",
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
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ]);

    (list, song_detail)
}

fn handle_song_end_conditions(
    track_player: &mut Player,
    click_player: &mut Player,
    songlist_state: &mut ListState,
    arguments: &PlayerArguments,
) -> Result<PlayerArguments, String> {
    
    if let Some(selected) = songlist_state.selected() {
        let amount_songs = read_songs().expect("can't fetch play list").len();
        if selected > amount_songs - 1 {
            songlist_state.select(Some(0));
        } else {
            songlist_state.select(Some(selected + 1));
        }
    }

    if let Some(selected) = songlist_state.selected() {
        let (track_file, click_file) = get_file_paths(&arguments.music_folder, selected + 1);

        let play_arguments = PlayerArguments {
            music_folder: arguments.music_folder.clone(),
            track_song: track_file,
            click_song: click_file,
            track_volume: arguments.track_volume,
            click_volume: arguments.click_volume,
            track_device_position: arguments.track_device_position,
            click_device_position: arguments.click_device_position,
            playback_speed: arguments.playback_speed,
        };

        let track_volume = Some(play_arguments.track_volume);
        let click_volume = Some(play_arguments.click_volume);

        let track_song =
            Song::from_file(&arguments.track_song, track_volume).expect("Could not create track song");
        let click_song =
            Song::from_file(&arguments.click_song, click_volume).expect("Could not create click song");

        track_player
            .play_song_now(&track_song, None)
            .expect("Could not play track song");
        click_player
            .play_song_now(&click_song, None)
            .expect("Could not play click song");

        Ok(play_arguments)

    } else {
        Err("Could not find next song".to_string())
    }
}

fn handle_q_event<T>(
    event: KeyEvent,
    track_player: &mut Player,
    click_player: &mut Player,
    terminal: &mut Terminal<T>,
) where
    T: Backend,
{
    if event.kind == KeyEventKind::Release {
        info!("Quitting");
        track_player.stop();
        click_player.stop();
        disable_raw_mode().expect("Can not disable raw mode");
        terminal.clear().expect("Failed to clear the terminal");
        terminal.show_cursor().expect("Failed to show cursor");
        std::process::exit(0);
    }
}
