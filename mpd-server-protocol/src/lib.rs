use async_trait::async_trait;
use bstr::{BStr, BString, ByteSlice, ByteVec};
use std::io::{Cursor, Write};
use std::ops::RangeInclusive;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tracing::{event, Level};
pub use url::Url;

/// MPD state: play, stop, or pause
#[derive(Copy, Clone, Debug)]
pub enum MPDState {
    Play,
    Stop,
    Pause,
}

impl Default for MPDState {
    fn default() -> Self {
        Self::Stop
    }
}

impl std::fmt::Display for MPDState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MPDState::Play => write!(f, "play"),
            MPDState::Stop => write!(f, "stop"),
            MPDState::Pause => write!(f, "pause"),
        }
    }
}

/// MPD status: reports the current status of the player and the volume level.
#[derive(Debug, Default)]
pub struct MPDStatus {
    pub volume: Option<u8>,
    pub repeat: Option<bool>,
    pub random: Option<bool>,
    pub single: Option<bool>,
    pub consume: Option<bool>,
    pub playlist: Option<u32>,
    pub playlistlength: Option<usize>,
    pub state: MPDState,
    pub song: Option<usize>,
    pub songid: Option<usize>,
    pub nextsong: Option<usize>,
    pub nextsongid: Option<usize>,
    pub elapsed: Option<Duration>,
    pub duration: Option<Duration>,
    // bitrate: Option<i32>,
    pub xfade: Option<usize>,
    pub mixrampdb: Option<f64>,
    pub mixrampdelay: Option<usize>,
    // audio
    // updating_db
    // error
}

impl std::fmt::Display for MPDStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "partition: default")?;
        if let Some(volume) = self.volume {
            writeln!(f, "volume: {}", volume)?
        }
        if let Some(repeat) = self.repeat {
            writeln!(f, "repeat: {}", repeat as usize)?
        }
        if let Some(random) = self.random {
            writeln!(f, "random: {}", random as usize)?
        }
        if let Some(single) = self.single {
            writeln!(f, "single: {}", single as usize)?;
        }
        if let Some(consume) = self.consume {
            writeln!(f, "consume: {}", consume as usize)?;
        }
        if let Some(playlist) = self.playlist {
            writeln!(f, "playlist: {}", playlist)?;
        }
        if let Some(playlistlength) = self.playlistlength {
            writeln!(f, "playlistlength: {}", playlistlength)?;
        }
        writeln!(f, "state: {}", self.state)?;
        if let Some(song) = self.song {
            writeln!(f, "song: {}", song)?;
        }
        if let Some(songid) = self.songid {
            writeln!(f, "songid: {}", songid)?;
        }
        if let Some(nextsong) = self.nextsong {
            writeln!(f, "nextsong: {}", nextsong)?;
        }
        if let Some(nextsongid) = self.nextsongid {
            writeln!(f, "nextsongid: {}", nextsongid)?;
        }
        if let Some(elapsed) = self.elapsed {
            if let Some(duration) = self.duration {
                writeln!(f, "time: {}:{}", elapsed.as_secs(), duration.as_secs())?;
            }
        }
        if let Some(elapsed) = self.elapsed {
            writeln!(f, "elapsed: {:.3}", elapsed.as_secs_f32())?;
        }
        if let Some(duration) = self.duration {
            writeln!(f, "duration: {:.3}", duration.as_secs_f32())?;
        }
        Ok(())
    }
}

impl MPDStatus {
    async fn send(
        &self,
        stream: &mut (impl AsyncWriteExt + Unpin),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = [0u8; 1024];
        let mut cursor = Cursor::new(&mut buf[..]);
        write!(cursor, "{}", self).unwrap();
        let data = &cursor.get_ref()[..(cursor.position() as usize)];
        stream.write_all(data).await?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct File {
    pub duration: Option<usize>,
    pub album: Option<String>,
    pub artist: Vec<String>,
    pub title: Option<String>,
    pub track: Option<usize>,
    pub year: Option<usize>,
    pub genre: Vec<String>,
}

impl File {
    fn write_to(&self, writer: &mut (dyn std::io::Write + Send + Sync)) {
        if let Some(duration) = self.duration {
            writeln!(writer, "duration: {}", duration).unwrap();
            writeln!(writer, "Time: {}", duration).unwrap();
        }
        for artist in &self.artist {
            writeln!(writer, "Artist: {}", artist).unwrap();
        }
        if let Some(ref album) = self.album {
            writeln!(writer, "Album: {}", album).unwrap();
        }
        if let Some(ref title) = self.title {
            writeln!(writer, "Title: {}", title).unwrap();
        }
        if let Some(track) = self.track {
            writeln!(writer, "Track: {}", track).unwrap();
        }
        if let Some(year) = self.year {
            writeln!(writer, "Date: {}", year).unwrap();
        }
        for genre in &self.genre {
            writeln!(writer, "Genre: {}", genre).unwrap();
        }
    }
}

pub struct DirEntry {
    pub path: PathBuf,
    pub file: Option<File>,
    pub last_modified: Option<SystemTime>,
}

impl DirEntry {
    fn write_to(&self, writer: &mut (dyn std::io::Write + Send + Sync)) {
        if let Some(ref file) = self.file {
            writer.write_all(b"file: ").unwrap();
            writer.write_all(self.path.as_os_str().as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
            file.write_to(writer);
        } else {
            writer.write_all(b"directory: ").unwrap();
            writer.write_all(self.path.as_os_str().as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }
}

pub struct QueueEntry {
    pub path: PathBuf,
    pub file: File,
    pub id: BString,
    pub position: usize,
}

impl QueueEntry {
    fn write_to(&self, writer: &mut (dyn std::io::Write + Send + Sync)) {
        writer.write_all(b"file: ").unwrap();
        writer.write_all(self.path.as_os_str().as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
        self.file.write_to(writer);
        writeln!(writer, "Pos: {}", self.position).unwrap();
        writeln!(writer, "Id: {}", self.id).unwrap();
    }
}

#[async_trait]
pub trait CommandHandler {
    // fn url_parse(input: &str) -> Url;

    async fn get_status(&mut self) -> MPDStatus;
    async fn list_directory(
        &mut self,
        path: Option<&Url>,
    ) -> Result<Vec<DirEntry>, Box<dyn std::error::Error + Send + Sync>>;

    /// Returns the current song in the queue.
    async fn queue_current(&mut self) -> Option<QueueEntry>;

    /// Returns a range of songs in the queue based on their position.
    async fn queue_list(&mut self, range: Option<RangeInclusive<usize>>) -> Vec<QueueEntry>;

    /// Returns a specific song in the queue based on its id.
    async fn queue_get(&mut self, id: &BStr) -> Option<QueueEntry>;

    /// Adds a song to the queue and return it's id.
    async fn queue_add_file(
        &mut self,
        path: &Url,
        pos: Option<usize>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// Adds a song to the queue and return it's id.
    async fn queue_add(
        &mut self,
        path: &Url,
        pos: Option<usize>,
    ) -> Result<Option<usize>, Box<dyn std::error::Error + Send + Sync>>;

    async fn queue_swap(
        &mut self,
        pos1: usize,
        pos2: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn queue_delete(
        &mut self,
        range: RangeInclusive<usize>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn queue_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn previous(&mut self);
    async fn play(&mut self, pos: usize);
    async fn playid(&mut self, id: usize);
    async fn next(&mut self);
    async fn stop(&mut self);
    async fn pause(&mut self, pause: Option<bool>);
    async fn random(&mut self, state: bool)
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn seek(
        &mut self,
        position: usize,
        time: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn seek_current(
        &mut self,
        time: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn volume_get(&mut self) -> usize;
    async fn volume_set(&mut self, level: usize);

    // Library APIs

    /// Updates the music database: find new files, remove deleted files, update modified files.
    /// `uri` is a particular directory or song/file to update. If you do not specify it, everything is updated.
    /// `rescan` should force rescan unmodified files.
    async fn library_update(
        &mut self,
        uri: Option<&Url>,
        rescan: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, PartialEq)]
enum TagTypes {
    All,
    Clear,
    Disable(Vec<BString>),
    Enable(Vec<BString>),
    List,
}

#[derive(Debug, PartialEq)]
enum MPDSubCommand {
    Add(Url),
    AddId(Url, Option<usize>),
    Clear,
    Commands,
    CurrentSong,
    Decoders,
    Delete(RangeInclusive<usize>),
    GetVol,
    Idle,
    Invalid {
        name: BString,
        args: BString,
        reason: CommandError,
    },
    ListPlaylist(BString),
    ListPlaylistInfo(BString),
    ListPlaylists,
    LsInfo(Option<Url>),
    Next,
    NoIdle,
    NotCommands,
    Outputs,
    Pause(Option<bool>),
    Play(usize),
    PlayId(usize),
    PlaylistChanges {
        version: usize,
        range: Option<RangeInclusive<usize>>,
    },
    PlaylistChangesPosId {
        version: usize,
        range: Option<RangeInclusive<usize>>,
    },
    PlaylistId(Option<BString>),
    PlaylistInfo(Option<RangeInclusive<usize>>),
    Previous,
    Random {
        state: bool,
    },
    Rescan {
        uri: Option<Url>,
    },
    Seek {
        position: usize,
        time: Duration,
    },
    SeekCurrent {
        time: Duration,
    },
    SetVol(usize),
    Status,
    Stats,
    Stop,
    Swap(usize, usize),
    TagTypes(TagTypes),
    Update {
        uri: Option<Url>,
    },
    UrlHandlers,
}

impl MPDSubCommand {
    fn name(&self) -> &BStr {
        <&BStr>::from(match self {
            Self::Add(_) => &b"add"[..],
            Self::AddId(..) => b"addid",
            Self::Clear => b"clear",
            Self::Commands => b"commands",
            Self::CurrentSong => b"currentsong",
            Self::Decoders => b"decoders",
            Self::Delete(_) => b"delete",
            Self::GetVol => b"getvol",
            Self::Idle => b"idle",
            Self::Invalid { name, .. } => name,
            Self::ListPlaylist(_) => b"listplaylist",
            Self::ListPlaylistInfo(_) => b"listplaylistinfo",
            Self::ListPlaylists => b"listplaylists",
            Self::LsInfo(_) => b"lsinfo",
            Self::Next => b"next",
            Self::NoIdle => b"noidle",
            Self::NotCommands => b"notcommands",
            Self::Outputs => b"outputs",
            Self::Pause(_) => b"pause",
            Self::Play(_) => b"play",
            Self::PlayId(_) => b"playid",
            Self::PlaylistChanges { .. } => b"plchanges",
            Self::PlaylistChangesPosId { .. } => b"plchangesposid",
            Self::PlaylistId(_) => b"playlistid",
            Self::PlaylistInfo(_) => b"playlistinfo",
            Self::Previous => b"previous",
            Self::Random { .. } => b"random",
            Self::Rescan { .. } => b"rescan",
            Self::Seek { .. } => b"seek",
            Self::SeekCurrent { .. } => b"seekcur",
            Self::SetVol(_) => b"setvol",
            Self::Status => b"status",
            Self::Stats => b"stats",
            Self::Stop => b"stop",
            Self::Swap(..) => b"swap",
            Self::TagTypes(_) => b"tagtypes",
            Self::Update { .. } => b"update",
            Self::UrlHandlers => b"urlhandlers",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CommandError {
    Unknown(String),
    InvalidArgument(String),
    NoExist(String),
}

impl CommandError {
    async fn send(
        &self,
        idx: usize,
        name: &BStr,
        stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (code, msg) = match self {
            CommandError::InvalidArgument(ref msg) => (2u8, msg),
            CommandError::Unknown(ref msg) => (5u8, msg),
            CommandError::NoExist(ref msg) => (50u8, msg),
        };
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
        writeln!(writer, "ACK [{}@{}] {{{}}} {}", code, idx, name, msg).unwrap();
        let data = &cursor.get_ref()[..(cursor.position() as usize)];
        eprintln!("Sending error: {:?}", String::from_utf8_lossy(data));
        stream.write_all(data).await?;
        Ok(())
    }
}

async fn lsinfo(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    url: Option<&Url>,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    let entries = match handler.list_directory(url).await {
        Ok(entries) => entries,
        Err(err) => return Ok(Err(CommandError::NoExist(err.to_string()))),
    };
    for entry in entries {
        entry.write_to(writer);
    }
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn currentsong(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    if let Some(queue) = handler.queue_current().await {
        queue.write_to(writer);
    }
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn getvol(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    writeln!(writer, "volume: {}", handler.volume_get().await).unwrap();
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn playlistid(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    id: Option<&BString>,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    if let Some(id) = id {
        if let Some(item) = handler.queue_get(id.as_bstr()).await {
            item.write_to(writer);
        }
    } else {
        for item in handler.queue_list(None).await {
            item.write_to(writer);
        }
    }
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn playlistinfo(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    range: Option<RangeInclusive<usize>>,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    for item in handler.queue_list(range).await {
        item.write_to(writer);
    }
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn add(
    _: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    url: &Url,
    _: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    match handler.queue_add(url, None).await {
        Ok(_) => Ok(Ok(())),
        Err(err) => Ok(Err(CommandError::NoExist(err.to_string()))),
    }
}

async fn addid(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    handler: &mut impl CommandHandler,
    url: &Url,
    position: Option<usize>,
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    let id = match handler.queue_add_file(url, position).await {
        Ok(id) => id,
        Err(err) => return Ok(Err(CommandError::NoExist(err.to_string()))),
    };
    buf.clear();
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    writeln!(writer, "Id: {}", id).unwrap();
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

async fn commands(
    stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
    buf: &mut Vec<u8>,
) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
    let mut cursor = Cursor::new(&mut *buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    writeln!(writer, "command: add")?;
    writeln!(writer, "command: addid")?;
    writeln!(writer, "command: addtagid")?;
    writeln!(writer, "command: albumart")?;
    writeln!(writer, "command: channels")?;
    writeln!(writer, "command: clear")?;
    writeln!(writer, "command: clearerror")?;
    writeln!(writer, "command: cleartagid")?;
    writeln!(writer, "command: close")?;
    writeln!(writer, "command: commands")?;
    writeln!(writer, "command: config")?;
    writeln!(writer, "command: consume")?;
    writeln!(writer, "command: count")?;
    writeln!(writer, "command: crossfade")?;
    writeln!(writer, "command: currentsong")?;
    writeln!(writer, "command: decoders")?;
    writeln!(writer, "command: delete")?;
    writeln!(writer, "command: deleteid")?;
    writeln!(writer, "command: delpartition")?;
    writeln!(writer, "command: disableoutput")?;
    writeln!(writer, "command: enableoutput")?;
    writeln!(writer, "command: find")?;
    writeln!(writer, "command: findadd")?;
    writeln!(writer, "command: getvol")?;
    writeln!(writer, "command: idle")?;
    writeln!(writer, "command: kill")?;
    writeln!(writer, "command: list")?;
    writeln!(writer, "command: listall")?;
    writeln!(writer, "command: listallinfo")?;
    writeln!(writer, "command: listfiles")?;
    writeln!(writer, "command: listmounts")?;
    writeln!(writer, "command: listpartitions")?;
    writeln!(writer, "command: listplaylist")?;
    writeln!(writer, "command: listplaylistinfo")?;
    writeln!(writer, "command: listplaylists")?;
    writeln!(writer, "command: load")?;
    writeln!(writer, "command: lsinfo")?;
    writeln!(writer, "command: mixrampdb")?;
    writeln!(writer, "command: mixrampdelay")?;
    writeln!(writer, "command: mount")?;
    writeln!(writer, "command: move")?;
    writeln!(writer, "command: moveid")?;
    writeln!(writer, "command: moveoutput")?;
    writeln!(writer, "command: newpartition")?;
    writeln!(writer, "command: next")?;
    writeln!(writer, "command: notcommands")?;
    writeln!(writer, "command: outputs")?;
    writeln!(writer, "command: outputset")?;
    writeln!(writer, "command: partition")?;
    writeln!(writer, "command: password")?;
    writeln!(writer, "command: pause")?;
    writeln!(writer, "command: ping")?;
    writeln!(writer, "command: play")?;
    writeln!(writer, "command: playid")?;
    writeln!(writer, "command: playlist")?;
    writeln!(writer, "command: playlistadd")?;
    writeln!(writer, "command: playlistclear")?;
    writeln!(writer, "command: playlistdelete")?;
    writeln!(writer, "command: playlistfind")?;
    writeln!(writer, "command: playlistid")?;
    writeln!(writer, "command: playlistinfo")?;
    writeln!(writer, "command: playlistmove")?;
    writeln!(writer, "command: playlistsearch")?;
    writeln!(writer, "command: plchanges")?;
    writeln!(writer, "command: plchangesposid")?;
    writeln!(writer, "command: previous")?;
    writeln!(writer, "command: prio")?;
    writeln!(writer, "command: prioid")?;
    writeln!(writer, "command: random")?;
    writeln!(writer, "command: rangeid")?;
    writeln!(writer, "command: readcomments")?;
    writeln!(writer, "command: readmessages")?;
    writeln!(writer, "command: readpicture")?;
    writeln!(writer, "command: rename")?;
    writeln!(writer, "command: repeat")?;
    writeln!(writer, "command: replay_gain_mode")?;
    writeln!(writer, "command: replay_gain_status")?;
    writeln!(writer, "command: rescan")?;
    writeln!(writer, "command: rm")?;
    writeln!(writer, "command: save")?;
    writeln!(writer, "command: search")?;
    writeln!(writer, "command: searchadd")?;
    writeln!(writer, "command: searchaddpl")?;
    writeln!(writer, "command: seek")?;
    writeln!(writer, "command: seekcur")?;
    writeln!(writer, "command: seekid")?;
    writeln!(writer, "command: sendmessage")?;
    writeln!(writer, "command: setvol")?;
    writeln!(writer, "command: shuffle")?;
    writeln!(writer, "command: single")?;
    writeln!(writer, "command: stats")?;
    writeln!(writer, "command: status")?;
    writeln!(writer, "command: stop")?;
    writeln!(writer, "command: subscribe")?;
    writeln!(writer, "command: swap")?;
    writeln!(writer, "command: swapid")?;
    writeln!(writer, "command: tagtypes")?;
    writeln!(writer, "command: toggleoutput")?;
    writeln!(writer, "command: unmount")?;
    writeln!(writer, "command: unsubscribe")?;
    writeln!(writer, "command: update")?;
    writeln!(writer, "command: urlhandlers")?;
    writeln!(writer, "command: volume")?;
    let data = &cursor.get_ref()[..(cursor.position() as usize)];
    stream.write_all(data).await?;
    Ok(Ok(()))
}

impl MPDSubCommand {
    async fn process(
        &self,
        stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
        handler: &mut impl CommandHandler,
        buf: &mut Vec<u8>,
    ) -> Result<Result<(), CommandError>, Box<dyn std::error::Error + Send + Sync>> {
        event!(Level::DEBUG, "Processing command: {:#?}", self);
        match self {
            Self::Add(url) => add(stream, handler, url, buf).await,
            Self::AddId(url, pos) => addid(stream, handler, url, pos.as_ref().copied(), buf).await,
            Self::Clear => {
                handler.queue_clear().await?;
                Ok(Ok(()))
            }
            Self::Commands => commands(stream, buf).await,
            Self::CurrentSong => currentsong(stream, handler, buf).await,
            Self::Decoders => Ok(Ok(())),
            Self::Delete(range) => {
                handler.queue_delete(range.clone()).await?;
                Ok(Ok(()))
            }
            Self::GetVol => getvol(stream, handler, buf).await,
            Self::Idle => {
                let cmd = parse_command_line(stream, buf).await?;
                if let Some(cmd) = cmd {
                    if cmd == MPDCommand::Sub(MPDSubCommand::NoIdle) {
                        Ok(Ok(()))
                    } else {
                        // stream.shutdown();
                        Ok(Err(CommandError::Unknown(String::from("invalid command"))))
                    }
                } else {
                    Ok(Ok(()))
                }
            }
            Self::Invalid { name, args, reason } => {
                eprintln!(
                    "Trying to process invalid command {:?} with {} args",
                    String::from_utf8_lossy(name),
                    args.len()
                );
                return Ok(Err(reason.clone()));
            }
            Self::ListPlaylist(_) => {
                return Ok(Err(CommandError::NoExist(
                    "playlist does not exist".to_owned(),
                )));
            }
            Self::ListPlaylistInfo(_) => {
                return Ok(Err(CommandError::NoExist(
                    "playlist does not exist".to_owned(),
                )));
            }
            Self::ListPlaylists => Ok(Ok(())),
            Self::LsInfo(path) => lsinfo(stream, handler, path.as_ref(), buf).await,
            Self::Next => {
                handler.next().await;
                Ok(Ok(()))
            }
            Self::NoIdle => Ok(Ok(())),
            Self::NotCommands => Ok(Ok(())),
            Self::Outputs => Ok(Ok(())),
            Self::Pause(pause) => {
                handler.pause(pause.as_ref().copied()).await;
                Ok(Ok(()))
            }
            Self::Play(pos) => {
                handler.play(*pos).await;
                Ok(Ok(()))
            }
            Self::PlayId(songid) => {
                handler.playid(*songid).await;
                Ok(Ok(()))
            }
            Self::PlaylistChanges { range, .. } => {
                playlistinfo(stream, handler, range.as_ref().cloned(), buf).await
            }
            Self::PlaylistChangesPosId { range, .. } => {
                playlistinfo(stream, handler, range.as_ref().cloned(), buf).await
            }
            Self::PlaylistId(id) => playlistid(stream, handler, id.as_ref(), buf).await,
            Self::PlaylistInfo(range) => {
                playlistinfo(stream, handler, range.as_ref().cloned(), buf).await
            }
            Self::Previous => {
                handler.previous().await;
                Ok(Ok(()))
            }
            Self::Random { state } => {
                handler.random(*state).await?;
                Ok(Ok(()))
            }
            Self::Rescan { uri } => {
                handler.library_update(uri.as_ref(), true).await?;
                Ok(Ok(()))
            }
            Self::Seek { position, time } => {
                handler.seek(*position, *time).await?;
                Ok(Ok(()))
            }
            Self::SeekCurrent { time } => {
                handler.seek_current(*time).await?;
                Ok(Ok(()))
            }
            Self::SetVol(level) => {
                handler.volume_set(*level).await;
                Ok(Ok(()))
            }
            Self::Status => {
                handler.get_status().await.send(stream).await?;
                Ok(Ok(()))
            }
            Self::Stats => Ok(Ok(())),
            Self::Stop => {
                handler.stop().await;
                Ok(Ok(()))
            }
            Self::Swap(pos1, pos2) => {
                handler.queue_swap(*pos1, *pos2).await?;
                Ok(Ok(()))
            }
            Self::TagTypes(_) => Ok(Ok(())),
            Self::Update { uri } => {
                handler.library_update(uri.as_ref(), false).await?;
                Ok(Ok(()))
            }
            Self::UrlHandlers => Ok(Ok(())),
        }
    }
}

trait FromBytes: Sized {
    type Err;

    fn from_bytes(s: &[u8]) -> Result<(Self, &[u8]), Self::Err>;
}

impl FromBytes for BString {
    type Err = BStringParseError;

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Self::Err> {
        let (first, rest) = if let Some((first, rest)) = bytes.split_first() {
            (*first, rest)
        } else {
            return Err(BStringParseError::Empty);
        };

        match first {
            b'"' | b'\'' => unescape_arg(first, rest),
            b' ' => Err(BStringParseError::Empty),
            _ => {
                if let Some(pos) = bytes.iter().position(|b| *b == b' ') {
                    let (left, right) = bytes.split_at(pos);
                    Ok((BString::from(left), &right[1..]))
                } else {
                    Ok((BString::from(bytes), &[]))
                }
            }
        }
    }
}

#[derive(Debug)]
enum BStringParseError {
    Empty,
    BadEscapeSequence,
    MissingClosingDelimiter(u8),
}

impl std::fmt::Display for BStringParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Invalid empty value"),
            Self::BadEscapeSequence { .. } => write!(f, "Bad escape sequence"),
            Self::MissingClosingDelimiter(c) => {
                write!(f, "Missing closing '{}'", <&BStr>::from(&[*c][..]))
            }
        }
    }
}

impl std::error::Error for BStringParseError {}

fn unescape_arg(delimiter: u8, arg: &[u8]) -> Result<(BString, &[u8]), BStringParseError> {
    let mut bytes = arg.iter();
    let mut out = Vec::with_capacity(arg.len());
    while let Some(c) = bytes.next() {
        match c {
            b'\\' => {
                if let Some(c) = bytes.next() {
                    out.push(*c);
                } else {
                    return Err(BStringParseError::BadEscapeSequence);
                }
            }
            _ => {
                if *c == delimiter {
                    return Ok((BString::from(out), bytes.as_slice()));
                } else {
                    out.push(*c);
                }
            }
        }
    }
    Err(BStringParseError::MissingClosingDelimiter(delimiter))
}

fn parse_integer(delimiter: u8, bytes: &[u8]) -> Result<(usize, &[u8]), IntParseError> {
    let (first, rest) = if let Some((first, rest)) = bytes.split_first() {
        (*first, rest)
    } else {
        return Err(IntParseError::Empty);
    };

    let mut num = usize::from(match first {
        b'0'..=b'9' => first - b'0',
        _ => {
            if first == delimiter {
                return Err(IntParseError::Empty);
            } else {
                return Err(IntParseError::InvalidDigit { num: None, pos: 0 });
            }
        }
    });

    for (i, b) in rest.iter().copied().enumerate() {
        match b {
            b'0'..=b'9' => {
                num = num.checked_mul(10).ok_or(IntParseError::Overflow)?;
                num = num
                    .checked_add(usize::from(b - b'0'))
                    .ok_or(IntParseError::Overflow)?;
            }
            _ => {
                if b == delimiter {
                    return Ok((num, &rest[(1 + i)..]));
                } else {
                    return Err(IntParseError::InvalidDigit {
                        num: Some(num),
                        pos: 1 + i,
                    });
                }
            }
        }
    }

    if delimiter == b' ' {
        Ok((num, &[]))
    } else {
        Err(IntParseError::MissingClosingDelimiter(delimiter))
    }
}

#[derive(Debug)]
enum IntParseError {
    Empty,
    InvalidDigit { num: Option<usize>, pos: usize },
    Overflow,
    MissingClosingDelimiter(u8),
}

impl std::fmt::Display for IntParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Invalid empty value"),
            Self::InvalidDigit { .. } => write!(f, "Invalid digit"),
            Self::Overflow => write!(f, "Integer overflow"),
            Self::MissingClosingDelimiter(c) => {
                write!(f, "Missing closing '{}'", <&BStr>::from(&[*c][..]))
            }
        }
    }
}

impl std::error::Error for IntParseError {}

impl FromBytes for usize {
    type Err = IntParseError;

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Self::Err> {
        if let Some((first, rest)) = bytes.split_first() {
            if *first == b'"' {
                parse_integer(b'"', rest)
            } else {
                parse_integer(b' ', bytes)
            }
        } else {
            Err(IntParseError::Empty)
        }
    }
}

impl FromBytes for RangeInclusive<usize> {
    type Err = IntParseError;

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Self::Err> {
        let (start, pos) = match usize::from_bytes(bytes) {
            Ok((num, rest)) => return Ok((num..=num, rest)),
            Err(IntParseError::InvalidDigit {
                num: Some(num),
                pos,
            }) if bytes[pos] == b':' => (num, pos + 1),
            Err(err) => return Err(err),
        };
        let (end, rest) = usize::from_bytes(&bytes[pos..])?;
        Ok((start..=end, rest))
    }
}

#[derive(Debug, PartialEq)]
enum MPDCommand {
    ListBegin {
        ok: bool,
        commands: Vec<MPDSubCommand>,
    },
    ListEnd,
    Sub(MPDSubCommand),
}

macro_rules! next_arg {
    ($name:ident, $args:ident, $arg_ty:ty) => {
        if $args.is_empty() {
            let msg = format!("wrong number of arguments for {:?}", $name);
            return MPDCommand::Sub(MPDSubCommand::Invalid {
                name: BString::from($name),
                args: BString::from(""),
                reason: CommandError::InvalidArgument(msg),
            });
        } else {
            match <$arg_ty>::from_bytes($args) {
                Ok((arg, rest)) => (arg, skip_whitespace(rest)),
                Err(err) => {
                    return MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from($name),
                        args: BString::from(""),
                        reason: CommandError::InvalidArgument(err.to_string()),
                    });
                }
            }
        }
    };
}

fn parse_command(name: &BStr, args: &[u8]) -> MPDCommand {
    let mut args = skip_whitespace(args);
    let cmd = if name.as_ref() == b"add" {
        let (input, rest) = next_arg!(name, args, BString);
        args = rest;
        let input = Vec::from(input).into_string_lossy();
        let base = Url::parse("file:///").unwrap();
        let opts = Url::options().base_url(Some(&base));
        match opts.parse(input.as_str()) {
            Ok(url) => MPDCommand::Sub(MPDSubCommand::Add(url)),
            Err(_) => {
                let msg = "Malformed URI".to_string();
                MPDCommand::Sub(MPDSubCommand::Invalid {
                    name: BString::from(name),
                    args: BString::from(args),
                    reason: CommandError::Unknown(msg),
                })
            }
        }
    } else if name.as_ref() == b"addid" {
        let (input, rest) = next_arg!(name, args, BString);
        args = rest;
        let position = if !args.is_empty() {
            let (pos, rest) = next_arg!(name, args, usize);
            args = rest;
            Some(pos)
        } else {
            None
        };
        let input = Vec::from(input).into_string_lossy();
        let base = Url::parse("file:///").unwrap();
        let opts = Url::options().base_url(Some(&base));
        match opts.parse(input.as_str()) {
            Ok(url) => MPDCommand::Sub(MPDSubCommand::AddId(url, position)),
            Err(_) => {
                let msg = "Malformed URI".to_string();
                MPDCommand::Sub(MPDSubCommand::Invalid {
                    name: BString::from(name),
                    args: BString::from(args),
                    reason: CommandError::Unknown(msg),
                })
            }
        }
    } else if name.as_ref() == b"clear" {
        MPDCommand::Sub(MPDSubCommand::Clear)
    } else if name.as_ref() == b"command_list_begin" {
        MPDCommand::ListBegin {
            ok: false,
            commands: Vec::new(),
        }
    } else if name.as_ref() == b"command_list_ok_begin" {
        MPDCommand::ListBegin {
            ok: true,
            commands: Vec::new(),
        }
    } else if name.as_ref() == b"command_list_end" {
        MPDCommand::ListEnd
    } else if name.as_ref() == b"commands" {
        MPDCommand::Sub(MPDSubCommand::Commands)
    } else if name.as_ref() == b"currentsong" {
        MPDCommand::Sub(MPDSubCommand::CurrentSong)
    } else if name.as_ref() == b"decoders" {
        MPDCommand::Sub(MPDSubCommand::Decoders)
    } else if name.as_ref() == b"delete" {
        let (arg, rest) = next_arg!(name, args, BString);
        args = rest;
        let range = RangeInclusive::from_bytes(arg.as_slice()).unwrap().0;
        MPDCommand::Sub(MPDSubCommand::Delete(range))
    } else if name.as_ref() == b"idle" {
        args = b"";
        MPDCommand::Sub(MPDSubCommand::Idle)
    } else if name.as_ref() == b"listplaylist" {
        let (playlist, rest) = next_arg!(name, args, BString);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::ListPlaylist(playlist))
    } else if name.as_ref() == b"listplaylistinfo" {
        let (playlist, rest) = next_arg!(name, args, BString);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::ListPlaylistInfo(playlist))
    } else if name.as_ref() == b"listplaylists" {
        MPDCommand::Sub(MPDSubCommand::ListPlaylists)
    } else if name.as_ref() == b"lsinfo" {
        if args.is_empty() {
            MPDCommand::Sub(MPDSubCommand::LsInfo(None))
        } else {
            let (input, rest) = next_arg!(name, args, BString);
            args = rest;
            let input = Vec::from(input).into_string_lossy();
            let base = Url::parse("file:///").unwrap();
            let opts = Url::options().base_url(Some(&base));
            match opts.parse(input.as_str()) {
                Ok(url) => MPDCommand::Sub(MPDSubCommand::LsInfo(Some(url))),
                Err(_) => {
                    let msg = "Malformed URI".to_string();
                    MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from(name),
                        args: BString::from(args),
                        reason: CommandError::Unknown(msg),
                    })
                }
            }
        }
    } else if name.as_ref() == b"next" {
        MPDCommand::Sub(MPDSubCommand::Next)
    } else if name.as_ref() == b"noidle" {
        MPDCommand::Sub(MPDSubCommand::NoIdle)
    } else if name.as_ref() == b"notcommands" {
        MPDCommand::Sub(MPDSubCommand::NotCommands)
    } else if name.as_ref() == b"outputs" {
        MPDCommand::Sub(MPDSubCommand::Outputs)
    } else if name.as_ref() == b"pause" {
        if args.is_empty() {
            MPDCommand::Sub(MPDSubCommand::Pause(None))
        } else {
            let (arg, rest) = next_arg!(name, args, usize);
            args = rest;
            match arg {
                0 => MPDCommand::Sub(MPDSubCommand::Pause(Some(false))),
                1 => MPDCommand::Sub(MPDSubCommand::Pause(Some(true))),
                _ => {
                    let msg = format!("Boolean (0/1) expected: {}", arg);
                    MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from(name),
                        args: BString::from(args),
                        reason: CommandError::Unknown(msg),
                    })
                }
            }
        }
    } else if name.as_ref() == b"play" {
        let (pos, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::Play(pos))
    } else if name.as_ref() == b"playid" {
        let (songid, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::PlayId(songid))
    } else if name.as_ref() == b"playlistid" {
        let id = if args.is_empty() {
            None
        } else {
            let (arg, rest) = next_arg!(name, args, BString);
            args = rest;
            Some(arg)
        };
        MPDCommand::Sub(MPDSubCommand::PlaylistId(id))
    } else if name.as_ref() == b"playlistinfo" {
        let range = if args.is_empty() {
            None
        } else {
            let (arg, rest) = next_arg!(name, args, BString);
            args = rest;
            Some(RangeInclusive::from_bytes(arg.as_slice()).unwrap().0)
        };
        MPDCommand::Sub(MPDSubCommand::PlaylistInfo(range))
    } else if name.as_ref() == b"plchanges" {
        let (version, rest) = next_arg!(name, args, usize);
        args = rest;
        let range = if args.is_empty() {
            None
        } else {
            let arg = BString::from_bytes(args).unwrap().0;
            Some(RangeInclusive::from_bytes(arg.as_slice()).unwrap().0)
        };
        MPDCommand::Sub(MPDSubCommand::PlaylistChanges { version, range })
    } else if name.as_ref() == b"plchangesposid" {
        let (version, rest) = next_arg!(name, args, usize);
        args = rest;
        let range = if args.is_empty() {
            None
        } else {
            let arg = BString::from_bytes(args).unwrap().0;
            Some(RangeInclusive::from_bytes(arg.as_slice()).unwrap().0)
        };
        MPDCommand::Sub(MPDSubCommand::PlaylistChangesPosId { version, range })
    } else if name.as_ref() == b"previous" {
        MPDCommand::Sub(MPDSubCommand::Previous)
    } else if name.as_ref() == b"random" {
        let (arg, rest) = next_arg!(name, args, usize);
        args = rest;
        match arg {
            0 => MPDCommand::Sub(MPDSubCommand::Random { state: false }),
            1 => MPDCommand::Sub(MPDSubCommand::Random { state: true }),
            _ => {
                let msg = format!("Boolean (0/1) expected: {}", arg);
                MPDCommand::Sub(MPDSubCommand::Invalid {
                    name: BString::from(name),
                    args: BString::from(args),
                    reason: CommandError::Unknown(msg),
                })
            }
        }
    } else if name.as_ref() == b"rescan" {
        if args.is_empty() {
            MPDCommand::Sub(MPDSubCommand::Rescan { uri: None })
        } else {
            let (input, rest) = next_arg!(name, args, BString);
            args = rest;
            let input = Vec::from(input).into_string_lossy();
            let base = Url::parse("file:///").unwrap();
            let opts = Url::options().base_url(Some(&base));
            match opts.parse(input.as_str()) {
                Ok(url) => MPDCommand::Sub(MPDSubCommand::Rescan { uri: Some(url) }),
                Err(_) => {
                    let msg = "Malformed URI".to_string();
                    MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from(name),
                        args: BString::from(args),
                        reason: CommandError::Unknown(msg),
                    })
                }
            }
        }
    } else if name.as_ref() == b"seek" {
        let (position, rest) = next_arg!(name, args, usize);
        args = rest;
        let (time, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::Seek {
            position,
            time: Duration::from_secs(time as u64),
        })
    } else if name.as_ref() == b"seekcur" {
        let (time, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::SeekCurrent {
            time: Duration::from_secs(time as u64),
        })
    } else if name.as_ref() == b"setvol" {
        let (arg, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::SetVol(arg))
    } else if name.as_ref() == b"status" {
        MPDCommand::Sub(MPDSubCommand::Status)
    } else if name.as_ref() == b"stats" {
        MPDCommand::Sub(MPDSubCommand::Stats)
    } else if name.as_ref() == b"stop" {
        MPDCommand::Sub(MPDSubCommand::Stop)
    } else if name.as_ref() == b"swap" {
        let (pos1, rest) = next_arg!(name, args, usize);
        args = rest;
        let (pos2, rest) = next_arg!(name, args, usize);
        args = rest;
        MPDCommand::Sub(MPDSubCommand::Swap(pos1, pos2))
    } else if name.as_ref() == b"tagtypes" {
        if args.is_empty() {
            MPDCommand::Sub(MPDSubCommand::TagTypes(TagTypes::List))
        } else {
            let (cmd, rest) = next_arg!(name, args, BString);
            args = rest;
            match cmd.as_slice() {
                b"all" => MPDCommand::Sub(MPDSubCommand::TagTypes(TagTypes::All)),
                b"clear" => MPDCommand::Sub(MPDSubCommand::TagTypes(TagTypes::Clear)),
                b"disable" => {
                    let mut tags = Vec::new();
                    while !args.is_empty() {
                        let (tag, rest) = next_arg!(name, args, BString);
                        tags.push(tag);
                        args = rest;
                    }
                    MPDCommand::Sub(MPDSubCommand::TagTypes(TagTypes::Disable(tags)))
                }
                b"enable" => {
                    let mut tags = Vec::new();
                    while !args.is_empty() {
                        let (tag, rest) = next_arg!(name, args, BString);
                        tags.push(tag);
                        args = rest;
                    }
                    MPDCommand::Sub(MPDSubCommand::TagTypes(TagTypes::Enable(tags)))
                }
                _ => {
                    let msg = "Unknown sub command".to_string();
                    MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from(name),
                        args: BString::from(args),
                        reason: CommandError::InvalidArgument(msg),
                    })
                }
            }
        }
    } else if name.as_ref() == b"update" {
        if args.is_empty() {
            MPDCommand::Sub(MPDSubCommand::Update { uri: None })
        } else {
            let (input, rest) = next_arg!(name, args, BString);
            args = rest;
            let input = Vec::from(input).into_string_lossy();
            let base = Url::parse("file:///").unwrap();
            let opts = Url::options().base_url(Some(&base));
            match opts.parse(input.as_str()) {
                Ok(url) => MPDCommand::Sub(MPDSubCommand::Update { uri: Some(url) }),
                Err(_) => {
                    let msg = "Malformed URI".to_string();
                    MPDCommand::Sub(MPDSubCommand::Invalid {
                        name: BString::from(name),
                        args: BString::from(args),
                        reason: CommandError::Unknown(msg),
                    })
                }
            }
        }
    } else if name.as_ref() == b"urlhandlers" {
        MPDCommand::Sub(MPDSubCommand::UrlHandlers)
    } else {
        let msg = format!("unknown command {:?}", name);
        return MPDCommand::Sub(MPDSubCommand::Invalid {
            name: BString::from(""),
            args: BString::from(args),
            reason: CommandError::Unknown(msg),
        });
    };

    if args.is_empty() {
        cmd
    } else {
        let msg = format!("wrong number of arguments for {:?}", name);
        MPDCommand::Sub(MPDSubCommand::Invalid {
            name: BString::from(name),
            args: BString::from(args),
            reason: CommandError::InvalidArgument(msg),
        })
    }
}

#[inline]
fn is_whitespace(b: &u8) -> bool {
    *b == b' '
}

#[inline]
fn skip_whitespace(input: &[u8]) -> &[u8] {
    let pos = input.iter().copied().take_while(is_whitespace).count();
    &input[pos..]
}

async fn parse_command_line(
    stream: &mut (impl AsyncBufReadExt + Unpin),
    buf: &mut Vec<u8>,
) -> Result<Option<MPDCommand>, Box<dyn std::error::Error + Send + Sync>> {
    buf.clear();
    let read = stream.read_until(b'\n', buf).await?;
    if read == 0 {
        return Ok(None);
    }

    if buf.last() != Some(&b'\n') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "failed to read line from stream",
        )
        .into());
    } else {
        buf.pop();
    }

    let (name, args) = if let Some(pos) = buf.iter().position(|b| *b == b' ') {
        let (left, right) = buf.split_at(pos);
        (left, &right[1..])
    } else {
        (&buf[..], &b""[..])
    };

    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some(parse_command(<&BStr>::from(name), args)))
}

impl MPDCommand {
    async fn parse(
        stream: &mut (impl AsyncBufReadExt + Unpin),
        buf: &mut Vec<u8>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let mut command = parse_command_line(stream, buf).await?;
        if let Some(Self::ListBegin {
            ref mut commands, ..
        }) = command
        {
            loop {
                match parse_command_line(stream, buf).await? {
                    Some(Self::ListBegin { ok, .. }) => {
                        let cmd = if ok {
                            "command_list_ok_begin"
                        } else {
                            "command_list_begin"
                        };
                        let msg = format!("unknown command {:?}", cmd);
                        commands.push(MPDSubCommand::Invalid {
                            name: BString::from(""),
                            args: BString::from(""),
                            reason: CommandError::Unknown(msg),
                        })
                    }
                    Some(Self::ListEnd) => break,
                    Some(Self::Sub(next_command)) => commands.push(next_command),
                    None => {}
                }
            }
        }
        Ok(command)
    }

    async fn process(
        &self,
        stream: &mut (impl AsyncBufReadExt + AsyncWriteExt + Unpin),
        handler: &mut impl CommandHandler,
        buf: &mut Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::ListBegin { ok, commands } => {
                for (i, command) in commands.iter().enumerate() {
                    match command.process(stream, handler, buf).await? {
                        Ok(()) => {
                            if *ok {
                                stream.write_all(b"list_OK\n").await?;
                            }
                        }
                        Err(err) => {
                            return err.send(i, command.name(), stream).await;
                        }
                    }
                }
                stream.write_all(b"OK\n").await?;
            }
            Self::Sub(command) => match command.process(stream, handler, buf).await? {
                Ok(()) => stream.write_all(b"OK\n").await?,
                Err(err) => {
                    err.send(0, command.name(), stream).await?;
                }
            },
            _ => panic!("Could not process command: {:?}", self),
        };
        Ok(())
    }
}

pub struct Server<S: AsyncBufReadExt + AsyncWriteExt + Unpin, H: CommandHandler> {
    stream: S,
    handler: H,
    line: Vec<u8>,
}

impl<S: AsyncBufReadExt + AsyncWriteExt + Unpin, H: CommandHandler> Server<S, H> {
    pub async fn new(
        stream: S,
        handler: H,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut server = Self {
            stream,
            handler,
            line: Vec::with_capacity(2048),
        };

        server.stream.write_all(b"OK MPD 0.22.0\n").await?;

        Ok(server)
    }

    pub async fn poll(&mut self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match MPDCommand::parse(&mut self.stream, &mut self.line).await {
            Ok(Some(command)) => {
                command
                    .process(&mut self.stream, &mut self.handler, &mut self.line)
                    .await?;
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

#[test]
fn test_parse_command_setvol() {
    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"50"),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"  50"),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"50  "),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"  50  "),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"\"50\""),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"  \"50\""),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"\"50\"  "),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"  \"50\"  "),
        MPDCommand::Sub(MPDSubCommand::SetVol(50)),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b""),
        MPDCommand::Sub(MPDSubCommand::Invalid {
            name: BString::from("setvol"),
            args: BString::from(""),
            reason: CommandError::InvalidArgument(
                "wrong number of arguments for \"setvol\"".to_owned()
            )
        }),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"50a"),
        MPDCommand::Sub(MPDSubCommand::Invalid {
            name: BString::from("setvol"),
            args: BString::from(""),
            reason: CommandError::InvalidArgument("Invalid digit".to_owned())
        }),
    );

    assert_eq!(
        parse_command(<&BStr>::from("setvol"), b"\"50a\""),
        MPDCommand::Sub(MPDSubCommand::Invalid {
            name: BString::from("setvol"),
            args: BString::from(""),
            reason: CommandError::InvalidArgument("Invalid digit".to_owned())
        }),
    );
}
