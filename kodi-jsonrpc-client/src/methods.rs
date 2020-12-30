macro_rules! define_method {
    ($( #[$attr:meta] )* $root:ident . $method:ident { $( $( #[$arg_attr:meta] )* $arg_name:ident : $arg_ty:ty ),* } -> $return_ty:ty) => {
        paste::paste! {
            #[derive(Debug, serde::Serialize)]
            $( #[$attr] )*
            pub struct [<$root $method>] {
                $($( #[$arg_attr] )* pub $arg_name: $arg_ty,)*
            }

            impl $crate::KodiMethod for [<$root $method>] {
                const NAME: &'static str = std::concat!(std::stringify!($root), ".", std::stringify!($method));
                type Response = $return_ty;
            }
        }
    };
}

// Application methods

define_method!(
    #[doc="Retrieves the values of the given properties"]
    Application.GetProperties {
        properties: enumset::EnumSet<crate::types::application::property::Name>
    } -> crate::types::application::property::Value
);

impl ApplicationGetProperties {
    pub fn all() -> Self {
        Self {
            properties: enumset::EnumSet::all(),
        }
    }
}

define_method!(
    #[doc="Quit application"]
    Application.Quit {} -> String
);

define_method!(
    #[doc="Toggle mute/unmute"]
    Application.SetMute {
        mute: crate::types::global::Toggle
    } -> bool
);

define_method!(
    #[doc="Set the current volume"]
    Application.SetVolume {
        volume: usize
    } -> usize
);

// Audio Library methods

define_method!(
    #[doc="Cleans the audio library from non-existent items"]
    AudioLibrary.Clean {
        showdialogs: bool
    } -> String
);

// AudioLibrary.Export

// AudioLibrary.GetAlbumDetails

// AudioLibrary.GetAlbums

// AudioLibrary.GetArtistDetails

// AudioLibrary.GetArtists

// AudioLibrary.GetGenres

// AudioLibrary.GetProperties

// AudioLibrary.GetRecentlyAddedAlbums

// AudioLibrary.GetRecentlyAddedSongs

// AudioLibrary.GetRecentlyPlayedAlbums

// AudioLibrary.GetRecentlyPlayedSongs

// AudioLibrary.GetRoles

// AudioLibrary.GetSongDetails

// AudioLibrary.GetSongs

define_method!(
    #[doc="Get all music sources, including unique ID"]
    #[derive(Default)]
    AudioLibrary.GetSources {
        #[serde(skip_serializing_if = "enumset::EnumSet::is_empty")]
        properties: enumset::EnumSet<crate::types::library::fields::Source>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limits: Option<crate::types::list::Limits>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sort: Option<crate::types::list::Sort>
    } -> AudioLibraryGetSourcesResponse
);

#[derive(Debug, serde::Deserialize)]
pub struct AudioLibraryGetSourcesResponse {
    pub limits: crate::types::list::LimitsReturned,
    #[serde(default)]
    pub sources: Vec<crate::types::library::details::Source>,
}

define_method!(
    #[doc="Scans the audio sources for new library items"]
    AudioLibrary.Scan {
        #[serde(skip_serializing_if = "Option::is_none")]
        directory: Option<String>,
        #[doc="Whether or not to show the progress bar or any other GUI dialog"]
        showdialogs: bool
    } -> String
);

// AudioLibrary.SetAlbumDetails

// AudioLibrary.SetArtistDetails

// AudioLibrary.SetSongDetails

// Files methods

define_method!(
    #[doc="Get the directories and files in the given directory"]
    #[derive(Default)]
    Files.GetDirectory {
        directory: String,
        media: crate::types::files::Media,
        #[serde(skip_serializing_if = "enumset::EnumSet::is_empty")]
        properties: enumset::EnumSet<crate::types::list::fields::Files>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limits: Option<crate::types::list::Limits>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sort: Option<crate::types::list::Sort>
    } -> FilesGetDirectoryResponse
);

impl FilesGetDirectory {
    pub fn all_properties(directory: String, media: crate::types::files::Media) -> Self {
        Self {
            directory,
            media,
            properties: enumset::EnumSet::all(),
            limits: None,
            sort: None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FilesGetDirectoryResponse {
    pub limits: crate::types::list::LimitsReturned,
    #[serde(default)]
    pub files: Vec<crate::types::list::item::File>,
}

define_method!(
    #[doc="Get details for a specific file"]
    #[derive(Default)]
    Files.GetFileDetails {
        file: String,
        media: crate::types::files::Media,
        #[serde(skip_serializing_if = "enumset::EnumSet::is_empty")]
        properties: enumset::EnumSet<crate::types::list::fields::Files>
    } -> FilesGetFileDetailsResponse
);

impl FilesGetFileDetails {
    pub fn all_properties(file: String, media: crate::types::files::Media) -> Self {
        Self {
            file,
            media,
            properties: enumset::EnumSet::all(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesGetFileDetailsResponse {
    FileDetails(crate::types::list::item::File),
}

// Files.GetSources

// Files.PrepareDownload

// Files.SetFileDetails

// JSONRPC methods

#[derive(Debug, serde::Deserialize)]
pub enum JSONRPCVersionResponse {
    #[serde(rename = "version")]
    Version {
        major: usize,
        minor: usize,
        patch: usize,
    },
}

define_method!(
    #[doc="Retrieve the JSON-RPC protocol version."]
    JSONRPC.Version {} -> JSONRPCVersionResponse
);

// Player methods

define_method!(
    #[doc="Returns all active players"]
    Player.GetActivePlayers {} -> Vec<crate::types::player::ActivePlayer>
);

define_method!(
    #[doc="Retrieves the currently played item"]
    Player.GetItem {
        #[serde(rename = "playerid")]
        id: u8,
        properties: enumset::EnumSet<crate::types::list::fields::All>
    } -> PlayerGetItemResponse
);

impl PlayerGetItem {
    pub fn all_properties(id: u8) -> Self {
        Self {
            id,
            properties: enumset::EnumSet::all(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerGetItemResponse {
    Item(crate::types::list::item::All),
}

// Player.GetPlayers

define_method!(
    #[doc="Retrieves the values of the given properties"]
    Player.GetProperties {
        #[serde(rename = "playerid")]
        id: u8,
        properties: enumset::EnumSet<crate::types::player::property::Name>
    } -> crate::types::player::property::Value
);

impl PlayerGetProperties {
    pub fn all(id: u8) -> Self {
        Self {
            id,
            properties: enumset::EnumSet::all(),
        }
    }
}

// Player.GetViewMode

define_method!(
    #[doc="Go to previous/next/specific item in the playlist"]
    Player.GoTo {
        #[serde(rename = "playerid")]
        id: u8,
        to: crate::types::player::GoTo
    } -> String
);

// Player.Move

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum PlayerOpenItem {
    PlaylistAt {
        #[serde(rename = "playlistid")]
        id: usize,
        position: usize,
    },
    // TODO: support other open mode
}

#[derive(Debug, Default, serde::Serialize)]
pub struct PlayerOpenOptions {
    #[serde(rename = "playername")]
    name: Option<String>,
    repeat: Option<bool>,
    // TODO: support other resume mode
    resume: bool,
    shuffled: Option<bool>,
}

define_method!(
    #[doc="Start playback of either the playlist with the given ID, a slideshow with the pictures from the given directory or a single file or an item from the database."]
    Player.Open {
        item: PlayerOpenItem,
        options: PlayerOpenOptions
    } -> String
);

define_method!(
    #[doc="Pauses or unpause playback and returns the new state"]
    Player.PlayPause {
        #[serde(rename = "playerid")]
        id: u8,
        play: crate::types::global::Toggle
    } -> crate::types::player::Speed
);

impl PlayerPlayPause {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            play: Default::default(),
        }
    }
}

// Player.Rotate

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerSeekStep {
    SmallForward,
    SmallBackward,
    BigForward,
    BigBackward,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerSeekMode {
    Percentage(f64),
    Time(crate::types::global::Time),
    Step(PlayerSeekStep),
    Seconds(isize),
}

define_method!(
    #[doc="Seek through the playing item"]
    Player.Seek {
        #[serde(rename = "playerid")]
        id: u8,
        value: PlayerSeekMode
    } -> PlayerSeekResponse
);

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct PlayerSeekResponse {
    pub percentage: f64,
    pub time: crate::types::global::Time,
    pub totaltime: crate::types::global::Time,
}

// Player.SetAudioStream

// Player.SetPartymode

// Player.SetRepeat

// Player.SetShuffle

// Player.SetSpeed

// Player.SetSubtitle

// Player.SetVideoStream

// Player.SetViewMode

define_method!(
    #[doc="Stops playback"]
    Player.Stop {
        #[serde(rename = "playerid")]
        id: u8
    } -> String
);

impl PlayerStop {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}

// Player.Zoom

// Playlist methods

define_method!(
    #[doc="Add item(s) to playlist"]
    Playlist.Add {
        #[serde(rename = "playlistid")]
        id: u8,
        item: Vec<crate::types::playlist::Item>
    } -> String
);

define_method!(
    #[doc="Clear playlist"]
    Playlist.Clear {
        #[serde(rename = "playlistid")]
        id: u8
    } -> String
);

define_method!(
    #[doc="Playlist.GetItems"]
    Playlist.GetItems {
        #[serde(rename = "playlistid")]
        id: u8,
        #[serde(skip_serializing_if = "enumset::EnumSet::is_empty")]
        properties: enumset::EnumSet<crate::types::list::fields::All>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limits: Option<crate::types::list::Limits>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sort: Option<crate::types::list::Sort>
    } -> PlaylistGetItemsResponse
);

impl PlaylistGetItems {
    pub fn all_properties(id: u8) -> Self {
        Self {
            id,
            properties: enumset::EnumSet::all(),
            limits: None,
            sort: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct PlaylistGetItemsResponse {
    #[serde(default)]
    pub items: Vec<crate::types::list::item::All>,
    pub limits: crate::types::list::LimitsReturned,
}

define_method!(
    #[doc="Returns all existing playlists"]
    Playlist.GetPlaylists {} -> Vec<crate::types::playlist::Playlist>
);

define_method!(
    #[doc="Retrieves the values of the given properties"]
    Playlist.GetProperties {
        #[serde(rename = "playlistid")]
        id: u8,
        #[serde(skip_serializing_if = "enumset::EnumSet::is_empty")]
        properties: enumset::EnumSet<crate::types::playlist::property::Name>
    } -> crate::types::playlist::property::Value
);

impl PlaylistGetProperties {
    pub fn all(id: u8) -> Self {
        Self {
            id,
            properties: enumset::EnumSet::all(),
        }
    }
}

define_method!(
    #[doc="Insert item(s) into playlist. Does not work for picture playlists (aka slideshows)."]
    Playlist.Insert {
        #[serde(rename = "playlistid")]
        id: u8,
        position: usize,
        item: Vec<crate::types::playlist::Item>
    } -> String
);

define_method!(
    #[doc="Remove item from playlist. Does not work for picture playlists (aka slideshows)."]
    Playlist.Remove {
        #[serde(rename = "playlistid")]
        id: u8,
        position: usize
    } -> String
);

define_method!(
    #[doc="Swap items in the playlist. Does not work for picture playlists (aka slideshows)."]
    Playlist.Swap {
        #[serde(rename = "playlistid")]
        id: u8,
        position1: usize,
        position2: usize
    } -> String
);
