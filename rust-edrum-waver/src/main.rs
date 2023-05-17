use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    fmt,
    fs,
    io,
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clap::{Arg, ArgMatches};
use csv;
use rodio::{
    self, 
    cpal, 
    source::Amplify, 
    cpal::traits::HostTrait,
    OutputStream,
    Device,
    Decoder,
    Sink,
    Source,
};
use sevenz_rust;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
    Terminal,
};
use lazy_static::lazy_static;


#[derive(Debug, serde::Deserialize, Clone)]
#[allow(unused_variables)]
struct Song {
    file_name: String,
    #[allow(dead_code)] genre: String,
    #[allow(dead_code)] year: String, 
    #[allow(dead_code)] artist: String,
    #[allow(dead_code)] song: String,
    #[allow(dead_code)] album: String,
    #[allow(dead_code)] length: String,
    #[allow(dead_code)] bpm: usize,
    folder: String,
}

fn main() {

    setupUi();

    let matches = clap::Command::new("eDrums Wav Player")
        .version("0.1")
        .arg(Arg::new("music_folder").long("music_folder").required(true).help("Where your music files are stored"))
        .arg(Arg::new("track").long("track").required(true).help("Position in the csv file to play"))
        .arg(Arg::new("track_volume").long("track_volume").required(false).default_value("100"))
        .arg(Arg::new("click_volume").long("click_volume").required(false).default_value("100"))
        .arg(Arg::new("track_device").long("track_device").required(false).default_value("1"))
        .arg(Arg::new("click_device").long("click_device").required(false).default_value("1"))
        .arg(Arg::new("combined").long("combined").required(false).default_value("1"))
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

/// Retrieves file paths for music files in a specified folder.
/// If the file does not exist, but a matching "7z" file does,
/// it will automatically decompress the 7z file for you.
///
/// # Arguments
///
/// * `music_folder` - A string slice representing the path to the music folder.
/// * `song_position` - An `usize` indicating the position of the desired song.
///
/// # Returns
///
/// A `Result` containing a tuple with the file paths of two music files, or an error message as a `String`.
/// If successful, the tuple contains two `String`s representing the file paths.
/// If unsuccessful, an error message is returned as a `String`.
///
/// # Example
///
/// ```rust
/// let result = get_file_paths("/path/to/music/folder", 0);
/// match result {
///     Ok((file1, file2)) => {
///         println!("File 1: {}", file1);
///         println!("File 2: {}", file2);
///     },
///     Err(error) => {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
fn get_file_paths(music_folder: &str, song_position: usize) -> Result<(String, String), String> {

    let mut reader = csv::Reader::from_path("assets/song_list.csv").unwrap();
    let headers = reader.headers();
    println!("{:?}", headers);

    let mut position = 1;
    for record in reader.deserialize() {
        let song: Song = record.unwrap();

        if position == song_position {
            let track_path_str = format!("{}/{}/{}.wav", music_folder, song.folder, song.file_name);
            let click_path_str = format!("{}/{}/{}_click.wav", music_folder, song.folder, song.file_name);

            let mut path = PathBuf::new();
            path.push(music_folder);
            path.push(&track_path_str);

            if !path.exists() {
                // if there's a 7z file with the same name, decompress it 
                 let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
                 if ! archive_path.exists() {
                    return Err(format!("Failed to find file or 7z archive for {}", archive_path.display()));
                 } 
                 println!("Decompressing file: {}", archive_path.display());

                 let mut output_folder = PathBuf::new();
                 output_folder.push(music_folder);
                 output_folder.push(song.folder);

                 sevenz_rust::decompress_file(&archive_path, output_folder).expect("Failed to decompress file");

            }

            check_file_existence(music_folder, &track_path_str)?;
            check_file_existence(music_folder, &click_path_str)?;

            return Ok((track_path_str, click_path_str));
        } else {
            position += 1;

        }
    }

    Err("Could not find song".to_string())

}

/// Checks the existence of a file in a specified folder.
///
/// # Arguments
///
/// * `folder_path` - A string slice representing the path to the folder.
/// * `file_name` - A string slice representing the name of the file to check.
///
/// # Returns
///
/// A `Result` indicating the result of the existence check.
/// If the file exists, `Ok(())` is returned.
/// If the file does not exist or encounters an error, an error message is returned as a `String`.
///
/// # Example
///
/// ```rust
/// let result = check_file_existence("/path/to/folder", "example.txt");
/// match result {
///     Ok(()) => {
///         println!("File exists.");
///     },
///     Err(error) => {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
fn check_file_existence(folder_path: &str, file_name: &str) -> Result<(), String> {
    let mut path = PathBuf::new();
    path.push(folder_path);
    path.push(file_name);

    if let Err(_) = fs::metadata(&path) {
        return Err(format!("File '{}' does not exist", path.display()));
    }
    Ok(())
}

/// Runs the program as determined by the main function
fn run(matches: &ArgMatches) -> Result<i32, String> {

    // Parse the arguments
    let music_folder = matches.get_one::<String>("music_folder").expect("No folder provided");
    
    let track_position = matches.get_one::<String>("track")
        .unwrap_or(&"1.0".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let (track_path_str, click_path_str) = match get_file_paths(music_folder, track_position) {
        Ok((track_path_str, click_path_str)) => (track_path_str, click_path_str),
        Err(err) => return Err(err),
    };
    
    println!("Playing track: {}", track_path_str);
    println!("Playing click: {}", click_path_str);

    let track_volume = matches.get_one::<String>("track_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let click_volume = matches.get_one::<String>("click_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let track_device_position = matches.get_one::<String>("track_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let click_device_position = matches.get_one::<String>("click_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let combined = matches.get_one::<String>("combined")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
    if available_devices.is_empty() {
        return Err("No output devices found".to_string());
    }

    // Check if the device positions are valid
    let num_devices = available_devices.len();
    if track_device_position > num_devices {
        return Err("Invalid track output device".to_string());
    }
    if click_device_position > num_devices {
        return Err("Invalid click output device".to_string());
    }

    let track = fs::File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
    let click = fs::File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

    let track_source = Decoder::new(io::BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
    let click_source = Decoder::new(io::BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;
    let track_source_amplify = track_source.amplify(track_volume);
    let click_source_amplify = click_source.amplify(click_volume);

    if combined == 1 {
        match play_combined(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[track_device_position]
        ) {
            Ok(_) => {},
            Err(err) => return Err(err),
        }
    
    } else {
        match play_separate(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[track_device_position],
            &available_devices[click_device_position],
            track_volume,
            click_volume
        ) {
            Ok(_) => {},
            Err(err) => return Err(err),

        }
    }

    Ok(0)

}

fn play_combined(track_source: Amplify<Decoder<io::BufReader<fs::File>>>, click_source: Amplify<Decoder<io::BufReader<fs::File>>>, device: &Device) -> Result<(), String> {
    let (_stream, stream_handle) = OutputStream::try_from_device(&device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let combined_source = track_source.mix(click_source);

    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(combined_source);
    sink.sleep_until_end();
    Ok(())
}

fn play_separate(track_source: Amplify<Decoder<io::BufReader<fs::File>>>, click_source: Amplify<Decoder<io::BufReader<fs::File>>>, track_device: &Device, click_device: &Device, track_volume: f32, click_volume: f32) -> Result<(), String> {
    
    let (_track_stream, track_stream_handle) = OutputStream::try_from_device(track_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let (_click_stream, click_stream_handle) = OutputStream::try_from_device(click_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;

    let track_sink = rodio::Sink::try_new(&track_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    let click_sink = rodio::Sink::try_new(&click_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    track_sink.set_volume(track_volume);
    click_sink.set_volume(click_volume);

    track_sink.append(track_source);
    click_sink.append(click_source);

    let track_thread = thread::spawn(move || {
        track_sink.sleep_until_end();
    });

    let click_thread = thread::spawn(move || {
        click_sink.sleep_until_end();

    }); 
    

    track_thread.join().expect("track thread panicked");
    click_thread.join().expect("click thread panicked");

    Ok(())
}








#[derive(Serialize, Deserialize, Clone)]
struct Playlist {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}


enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
    #[error("error parsing the CsV file: {0}")]
    CsvError(#[from] csv::Error),
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Playlists,
    Songs,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Playlists => 1,
            MenuItem::Songs => 2,
        }
    }
}

// see: https://github.com/zupzup/rust-commandline-example/blob/main/src/main.rs

fn setupUi() -> Result<(), Box<dyn std::error::Error>> {
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

            let copyright = Paragraph::new("Drum karaoke player by Dennis Gurnick")
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
                }
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

fn read_playlists() -> Result<Vec<Playlist>, Error> {
    let db_content = fs::read_to_string("assets/playlists.json")?;
    let parsed: Vec<Playlist> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

lazy_static! {
    static ref SONGS: Mutex<Vec<Song>> = Mutex::new(Vec::new());
}

fn read_songs() -> Result<Vec<Song>, Error> {

    let mut songs = SONGS.lock().unwrap();

    if songs.is_empty() {
        let db_content = fs::read_to_string("assets/song_list.csv")?;
        let mut reader = csv::Reader::from_reader(db_content.as_bytes());

        for result in reader.deserialize() {
            let song: Song = result?;
            songs.push(song);
        }
    }

    Ok(songs.clone())

}
