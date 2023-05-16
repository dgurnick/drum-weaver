Rust wav player for drumming

Completely based on files and spreadsheet first listed on https://www.reddit.com/r/edrums/comments/1162lyh/sharing_my_library_of_5000_drumless_songs_with/

Files:    aHR0cHM6Ly9tZWdhLm56L2ZvbGRlci9mVjEzbFR6SyNwWndtTV82YzI2U3hWMjdIa3VVRjZB
Spreadsheet : aHR0cHM6Ly9kb2NzLmdvb2dsZS5jb20vc3ByZWFkc2hlZXRzL2QvMXY2N3dUNDJBNGxSanM3al9nbVJmdUV6ZkZDMUVnUk1leUtfeEt0YVBUMW8vZWRpdD91c3A9c2hhcmluZw==

Running it
cargo run -- --music_folder D: --track 356 --track_volume 100 --click_volume 80 --track_device 1 --click_device 1 --combined 0
where
1. target_folder is where you downloaded the massive amount of 7z files
2. track is the position in the csv file (minus 1)
3. track_volume is the master volume. In combined mode, it's what is used. In non-combined mode, it's the main track
4. click_volume is ignored when combined otherwise it's the volume of the click on it's device
5. track_device is where to play the main file in the collection of devices iterated
6. click_device is like the track_device
7. combined indicates if you want to play everything in one device (in case threading is crap) or separate

Tasks
[x] Basic player
[ ] Basic UI or can be as a local webserver (leptos?)
[ ] search and filter option
[ ] playlists
[ ] midi controller. something basic with a synth will do. For restart, skip song, skip forward and backward, etc
