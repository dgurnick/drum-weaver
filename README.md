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
2. execute ```rust-edrum-waver```
3. Note: the UI automatically moves to the next song. If you want to disable this, add ```--autoskip=0``` as a parameter when you run the app.

## Improvements
1. [x] Basic player
2. [x] Basic UI
3. [c] search and filter option
4. [ ] playlists
5. [ ] midi controller. something basic with a synth will do. For restart, skip song, skip forward and backward, etc
