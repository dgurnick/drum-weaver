use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
    Terminal,
};

//use lazy_static::lazy_static;

use crate::common::{Event, PlayerArguments, get_file_paths};
use crate::common::MenuItem;
use crate::common::read_playlists;
use crate::common::read_songs;
use playback_rs::{Player, Song};    

pub fn run_ui(arguments: PlayerArguments) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("Can not run in raw mode");
    
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

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

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear();

    let menu_titles = vec!["Home", "Playlists", "Songs", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut playlist_state = ListState::default();
    playlist_state.select(Some(0));

    let mut songlist_state = ListState::default();
    songlist_state.select(Some(0));

    let track_player = Player::new(None)?;
    let click_player = Player::new(None)?;

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
                }
            }
            rect.render_widget(copyright, chunks[2]);

        })?;

        match rx.recv()? {

            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode().expect("Can not disable raw mode");
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                },
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Playlists,
                KeyCode::Char('s') => active_menu_item = MenuItem::Songs,
                KeyCode::Down => {
                    
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {
                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected >= amount_playlists - 1 {
                                        playlist_state.select(Some(0));
                                    } else {
                                        playlist_state.select(Some(selected + 1));
                                    }
                                }

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

                            },
                            _ => {}

                        }

                    }
                }
                KeyCode::Up => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {

                                if let Some(selected) = playlist_state.selected() {
                                    let amount_playlists = read_playlists().expect("can't fetch play list").len();
                                    if selected > 0 {
                                        playlist_state.select(Some(selected - 1));
                                    } else {
                                        playlist_state.select(Some(amount_playlists - 1));
                                    }
                                }
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

                            },
                            _ => {}

                        }

                    }
                },
                KeyCode::Enter => {
                    if event.kind == KeyEventKind::Release {

                        match active_menu_item {
                            MenuItem::Playlists => {

                                println!("TODO Play selected play list");

                            },
                            MenuItem::Songs => {

                                if let Some(selected) = songlist_state.selected() {
                                    let (track_path, click_path) = get_file_paths(&arguments.music_folder, selected);
                                    let track_song = Song::from_file(&track_path, None)?;
                                    let click_song = Song::from_file(&click_path, None)?;
                                    track_player.play_song_now(&track_song, None)?;
                                    click_player.play_song_now(&click_song, None)?;

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