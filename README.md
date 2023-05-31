# Click & Main track player for drumming karaoke things (that I can't explain better)
> Written in Rust because why not?

## Overview
1. Based on files and spreadsheet first listed on [reddit](https://www.reddit.com/r/edrums/comments/1162lyh/sharing_my_library_of_5000_drumless_songs_with/)
   1. Files: ```aHR0cHM6Ly9tZWdhLm56L2ZvbGRlci9mVjEzbFR6SyNwWndtTV82YzI2U3hWMjdIa3VVRjZB```
   2. Spreadsheet : ```aHR0cHM6Ly9kb2NzLmdvb2dsZS5jb20vc3ByZWFkc2hlZXRzL2QvMXY2N3dUNDJBNGxSanM3al9nbVJmdUV6ZkZDMUVnUk1leUtfeEt0YVBUMW8vZWRpdD91c3A9c2hhcmluZw==```
2. Allows you to indicate which sound device you want the main and click tracks played out of
   1. Typical use-case: click in the headphones, main for the audience.
3. Tested on Windows (my music workstation; sorry) but probably works on other platforms.
4. The spreadsheet is included here as a CSV in the assets folder. 
5. Automatically extracts the 7z file if not already done. (Who wants to do that manually?)
6. Is peasant-level Rust. ymmv
7. Some songs will have clicks out of sync. This is because the dude who made all of these made some mistakes. Blame him and his AI bot. In all their glory.

## Running it
1. Download all the files
2. execute ```rust-edrum-waver --music_folder D:\drumless``` ... or point it to wherever you have the huge number of files saved.
3. Note: the UI automatically moves to the next song. If you want to disable this, add ```--autoskip=0``` as a parameter when you run the app.

## Using the UI
The UI is a very basic terminal app. It's not meant to do anything fancy (doesn't have search/filter/playlists yet) other than plan a song you want.

Shortcut "S": This brings up the list of songs found by the app.  You can use up/down arrow keys or pgup/pgdn to move up or down.
Shortcut "Enter": Clicking the ENTER key on a song plays it. It might take a few seconds if the file has not yet been decompressed. Be patient.
Shortcut "Space": Stops playing the current song.
Shortcut "D": here you can choose what audio output to use for what channel. up/down moves across devices. "c" selects it for the click track. "t" for the master track. This is super useful for drum karaoke!
Shortcut "left arrow": reduces playback speed if you're a shitty drummer
Shortcut "right arrow": makes maximum chaos. 
Shortcut "r": during playback resets the playback speed.

## Parameters:
1. **music_folder** is where you downloaded the massive amount of 7z files
2. **track** is the position in the csv file (minus 1). Useful really in the cli version and ignored in the UI
3. **track_volume** is the master volume.
4. **click_volume** is the click volume. For live performances, this is useful --I hope.
5. **track_device** is where to play the main file in the collection of devices iterated
6. **click_device** is like the track_device
7. **ui** If you just want to play the song, 0. If you want to navigate, 1.
8. **playback_speed** is the startup speed of the player.
9. **print_devices** will just print your audio devices and exit. You get the same thing in the UI by pressing "d"
10. **auto_skip** automatically moves to the next song in sequence. It defaults to skip and will not skip if any other parameter is passed. I'm a shitty developer.

## Improvements
1. [x] Basic player
2. [x] Basic UI
3. [c] search and filter option
4. [ ] playlists
5. [ ] midi controller. something basic with a synth will do. For restart, skip song, skip forward and backward, etc
