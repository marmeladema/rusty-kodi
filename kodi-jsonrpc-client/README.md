# kodi-jsonrpc-client

This is a work-in-progress implementation of rust "bindings" for [Kodi](https://kodi.tv) [JSONRPC API](https://kodi.wiki/view/JSON-RPC_API) using async networking.

It currently focus mostly on accessing audio related APIs since it has been developped for the `kodi-mpd-proxy` crate.

## Supported transport protocols

- [x] HTTP (using [reqwest](https://github.com/seanmonstar/reqwest))
- [ ] TCP
- [ ] WebSocket

## Supported methods

### Addons namespace

- [ ] Addons.ExecuteAddon
- [ ] Addons.GetAddonDetails
- [ ] Addons.GetAddons
- [ ] Addons.SetAddonEnabled

### Application namespace

- [x] Application.GetProperties
- [x] Application.Quit
- [x] Application.SetMute
- [x] Application.SetVolume

### AudioLibrary namespace

- [x] AudioLibrary.Clean
- [ ] AudioLibrary.Export
- [ ] AudioLibrary.GetAlbumDetails
- [x] AudioLibrary.GetAlbums
- [ ] AudioLibrary.GetArtistDetails
- [x] AudioLibrary.GetArtists
- [ ] AudioLibrary.GetGenres
- [ ] AudioLibrary.GetProperties
- [ ] AudioLibrary.GetRecentlyAddedAlbums
- [ ] AudioLibrary.GetRecentlyAddedSongs
- [ ] AudioLibrary.GetRecentlyPlayedAlbums
- [ ] AudioLibrary.GetRecentlyPlayedSongs
- [ ] AudioLibrary.GetRoles
- [ ] AudioLibrary.GetSongDetails
- [x] AudioLibrary.GetSongs
- [x] AudioLibrary.GetSources
- [x] AudioLibrary.Scan
- [ ] AudioLibrary.SetAlbumDetails
- [ ] AudioLibrary.SetArtistDetails
- [ ] AudioLibrary.SetSongDetails

### Favourites namespace

- [ ] Favourites.AddFavourite
- [ ] Favourites.GetFavourites

### Files namespace

- [x] Files.GetDirectory
- [x] Files.GetFileDetails
- [x] Files.GetSources
- [ ] Files.PrepareDownload
- [ ] Files.SetFileDetails

### GUI namespace

- [ ] GUI.ActivateWindow
- [ ] GUI.GetProperties
- [ ] GUI.GetStereoscopicModes
- [ ] GUI.SetFullscreen
- [ ] GUI.SetStereoscopicMode
- [ ] GUI.ShowNotification

### Input namespace

- [ ] Input.Back
- [ ] Input.ContextMenu
- [ ] Input.Down
- [ ] Input.ExecuteAction
- [ ] Input.Home
- [ ] Input.Info
- [ ] Input.Left
- [ ] Input.Right
- [ ] Input.Select
- [ ] Input.SendText
- [ ] Input.ShowCodec
- [ ] Input.ShowOSD
- [ ] Input.ShowPlayerProcessInfo
- [ ] Input.Up

### JSONRPC namespace

- [ ] JSONRPC.Introspect
- [ ] JSONRPC.NotifyAll
- [ ] JSONRPC.Permission
- [ ] JSONRPC.Ping
- [x] JSONRPC.Version

### PVR namespace

- [ ] PVR.AddTimer
- [ ] PVR.DeleteTimer
- [ ] PVR.GetBroadcastDetails
- [ ] PVR.GetBroadcasts
- [ ] PVR.GetChannelDetails
- [ ] PVR.GetChannelGroupDetails
- [ ] PVR.GetChannelGroups
- [ ] PVR.GetChannels
- [ ] PVR.GetProperties
- [ ] PVR.GetRecordingDetails
- [ ] PVR.GetRecordings
- [ ] PVR.GetTimerDetails
- [ ] PVR.GetTimers
- [ ] PVR.Record
- [ ] PVR.Scan
- [ ] PVR.ToggleTimer

### Player namespace

- [x] Player.GetActivePlayers
- [x] Player.GetItem
- [x] Player.GetPlayers
- [x] Player.GetProperties
- [ ] Player.GetViewMode
- [x] Player.GoTo
- [ ] Player.Move
- [x] Player.Open
- [x] Player.PlayPause
- [ ] Player.Rotate
- [x] Player.Seek
- [ ] Player.SetAudioStream
- [x] Player.SetPartymode
- [x] Player.SetRepeat
- [x] Player.SetShuffle
- [x] Player.SetSpeed
- [ ] Player.SetSubtitle
- [ ] Player.SetVideoStream
- [ ] Player.SetViewMode
- [x] Player.Stop
- [ ] Player.Zoom

### Player namespace

- [x] Playlist.Add
- [x] Playlist.Clear
- [x] Playlist.GetItems
- [x] Playlist.GetPlaylists
- [x] Playlist.GetProperties
- [x] Playlist.Insert
- [x] Playlist.Remove
- [x] Playlist.Swap

### Profiles namespace

- [ ] Profiles.GetCurrentProfile
- [ ] Profiles.GetProfiles
- [ ] Profiles.LoadProfile

### Settings namespace

- [ ] Settings.GetCategories
- [ ] Settings.GetSections
- [ ] Settings.GetSettingValue
- [ ] Settings.GetSettings
- [ ] Settings.ResetSettingValue
- [ ] Settings.SetSettingValue

### System namespace

- [ ] System.EjectOpticalDrive
- [ ] System.GetProperties
- [ ] System.Hibernate
- [ ] System.Reboot
- [ ] System.Shutdown
- [ ] System.Suspend

### Textures namespace

- [ ] Textures.GetTextures
- [ ] Textures.RemoveTexture

### VideoLibrary namespace

- [ ] VideoLibrary.Clean
- [ ] VideoLibrary.Export
- [ ] VideoLibrary.GetEpisodeDetails
- [ ] VideoLibrary.GetEpisodes
- [ ] VideoLibrary.GetGenres
- [ ] VideoLibrary.GetInProgressTVShows
- [ ] VideoLibrary.GetMovieDetails
- [ ] VideoLibrary.GetMovieSetDetails
- [ ] VideoLibrary.GetMovieSets
- [ ] VideoLibrary.GetMovies
- [ ] VideoLibrary.GetMusicVideoDetails
- [ ] VideoLibrary.GetMusicVideos
- [ ] VideoLibrary.GetRecentlyAddedEpisodes
- [ ] VideoLibrary.GetRecentlyAddedMovies
- [ ] VideoLibrary.GetRecentlyAddedMusicVideos
- [ ] VideoLibrary.GetSeasonDetails
- [ ] VideoLibrary.GetSeasons
- [ ] VideoLibrary.GetTVShowDetails
- [ ] VideoLibrary.GetTVShows
- [ ] VideoLibrary.GetTags
- [ ] VideoLibrary.RefreshEpisode
- [ ] VideoLibrary.RefreshMovie
- [ ] VideoLibrary.RefreshMusicVideo
- [ ] VideoLibrary.RefreshTVShow
- [ ] VideoLibrary.RemoveEpisode
- [ ] VideoLibrary.RemoveMovie
- [ ] VideoLibrary.RemoveMusicVideo
- [ ] VideoLibrary.RemoveTVShow
- [ ] VideoLibrary.Scan
- [ ] VideoLibrary.SetEpisodeDetails
- [ ] VideoLibrary.SetMovieDetails
- [ ] VideoLibrary.SetMovieSetDetails
- [ ] VideoLibrary.SetMusicVideoDetails
- [ ] VideoLibrary.SetSeasonDetails
- [ ] VideoLibrary.SetTVShowDetails

### XBMC namespace

- [ ] XBMC.GetInfoBooleans
- [ ] XBMC.GetInfoLabels

## Supported notifications

None :(

Notifications are mostly interesting to passively retrieve events from Kodi to the client. However it cannot work with the HTTP transport protocol, which is the only transport protocol currently supported.

## Repository layout

- `src/types.rs` contains the definition of all Kodi global types
- `src/methods.rs` contains the definitoon of all Kodi methods
- `src/lib.rs` contains main types and trait to manipulate access Kodi
- `src/main.rs` is a standalone executable to trigger a set of pre-defined JSONRPC methods calls and print the result. It's mainly used a quick'n'dirty tool to test the crate.

## Usage

```Rust
let client = reqwest::Client::new();
let client = KodiClient::new(client, reqwest::Url::parse("http://192.168.0.1:8989/jsonrpc")?);

let resp = client.send_method(JSONRPCVersion {}).await?;
event!(Level::INFO, "JSONRPC.Version:\n{:#?}", resp);
```

## TODO

- A test framework
- Avoid owned heap allocated data types in method parameters
- More complete methods coverage
- Support for other transport protocols
- Notifications ...
- Documentation