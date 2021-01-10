use async_trait::async_trait;
use bstr::{BStr, BString};
use clap::Clap;
use enum_map::EnumMap;
use enumset::EnumSet;
use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::types::list::item::FileType as KodiFileType;
use kodi_jsonrpc_client::KodiClient;
use mpd_server_protocol::{
    CommandHandler, LibraryEntry, MPDState, MPDStatus, MPDSubsystem, QueueEntry, QueueSong, Server,
    Song, Tag, TagFilter, TagType, Url,
};
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{event, Level};

mod player;

struct KodiProxyCommandHandler {
    kodi_client: KodiClient,
    player: Arc<player::KodiPlayer>,
    subsystem_events: EnumMap<MPDSubsystem, usize>,
    subsystem_notifier: watch::Receiver<usize>,
    subsystem_version: usize,
    tags: EnumSet<TagType>,
}

impl KodiProxyCommandHandler {
    fn new(
        kodi_client: KodiClient,
        player: Arc<player::KodiPlayer>,
        subsystem_notifier: watch::Receiver<usize>,
    ) -> Self {
        Self {
            kodi_client,
            player,
            subsystem_events: EnumMap::default(),
            subsystem_notifier,
            subsystem_version: 0,
            tags: EnumSet::all(),
        }
    }

    async fn path_remap(&self, path: &Path) -> Option<PathBuf> {
        let sources = self
            .kodi_client
            .send_method(AudioLibraryGetSources::default())
            .await
            .unwrap()
            .sources;
        for source in sources {
            let base = Path::new(OsStr::from_bytes(source.label.as_bytes()));
            if let Ok(rest) = path.strip_prefix(base) {
                let mut path = PathBuf::from(&source.file);
                path.push(rest);
                return Some(path);
            }
        }
        None
    }

    async fn song_id_to_pos(&self, songid: usize) -> Option<usize> {
        for (pos, item) in self.player.playlist_items().await.iter().enumerate().rev() {
            if item.id == Some(songid) {
                return Some(pos);
            }
        }
        None
    }

    fn events(&mut self, wanted: EnumSet<MPDSubsystem>) -> EnumSet<MPDSubsystem> {
        let mut set = EnumSet::empty();
        for (variant, value) in self.subsystem_events.iter_mut() {
            if wanted.contains(variant) {
                let count = self.player.event_get(variant);
                if count > *value {
                    *value = count;
                    set.insert(variant);
                }
            }
        }
        set
    }
}

fn usize_to_bstring(val: usize) -> BString {
    let mut buf = Vec::with_capacity(20);
    let mut cursor = std::io::Cursor::new(&mut buf);
    let writer = &mut cursor as &mut (dyn std::io::Write + Send + Sync);
    write!(writer, "{}", val).unwrap();
    BString::from(buf)
}

#[async_trait]
impl CommandHandler for KodiProxyCommandHandler {
    async fn get_status(&mut self) -> MPDStatus {
        let mut status = MPDStatus {
            volume: self.player.volume().await,
            ..Default::default()
        };
        let PlayerGetItemResponse::Item(item) = self
            .kodi_client
            .send_method(PlayerGetItem::all_properties(self.player.id()))
            .await
            .unwrap();
        if let Some(playlist_id) = self.player.playlist().await {
            let playlist_props = self
                .kodi_client
                .send_method(PlaylistGetProperties::all(playlist_id))
                .await
                .unwrap();
            status.playlistlength = playlist_props.size;
        }
        if self.player.position().await.is_some() {
            if let Some(speed) = self.player.speed().await {
                if speed == 0 {
                    status.state = MPDState::Pause;
                } else {
                    status.state = MPDState::Play;
                }
            }
        }
        status.random = self.player.shuffled().await;
        status.song = self.player.position().await;
        status.songid = item.id;
        status.elapsed = self.player.time().await;
        status.duration = self.player.totaltime().await;
        status.playlist = Some(self.player.event_get(MPDSubsystem::Playlist));
        status
    }

    async fn list_directory(
        &mut self,
        url: Option<&Url>,
    ) -> Result<Vec<LibraryEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self
            .kodi_client
            .send_method(AudioLibraryGetSources::default())
            .await?;

        let path = if let Some(url) = url {
            if url.scheme() != "file" {
                return Err("Unsupported URI scheme".into());
            }
            url.to_file_path().unwrap()
        } else {
            PathBuf::default()
        };
        if path == Path::new("/") || path == Path::new("") {
            Ok(resp
                .sources
                .into_iter()
                .map(|source| LibraryEntry::Directory {
                    path: PathBuf::from(source.label),
                    last_modified: None,
                })
                .collect())
        } else {
            for source in resp.sources {
                let base = Path::new(OsStr::from_bytes(source.label.as_bytes()));
                if let Ok(rest) = path.strip_prefix("/").unwrap().strip_prefix(base) {
                    let mut path = PathBuf::from(&source.file);
                    path.push(rest);
                    let entries = self
                        .kodi_client
                        .send_method(FilesGetDirectory::all_properties(
                            path.to_str().unwrap().to_owned(),
                            kodi_jsonrpc_client::types::files::Media::Music,
                        ))
                        .await?;
                    return Ok(entries
                        .files
                        .into_iter()
                        .map(move |file| {
                            let source_path = Path::new(OsStr::from_bytes(source.file.as_bytes()));
                            let rest = Path::new(OsStr::from_bytes(file.file.as_bytes()))
                                .strip_prefix(source_path)
                                .unwrap();
                            let mut path = PathBuf::from(&source.label);
                            path.push(rest);
                            match file.filetype {
                                KodiFileType::Directory => LibraryEntry::Directory {
                                    path,
                                    last_modified: None,
                                },
                                KodiFileType::File => LibraryEntry::File(Song {
                                    path,
                                    last_modified: None,
                                    format: None,
                                    duration: file.duration,
                                    tags: {
                                        let mut vec = Vec::new();
                                        vec.extend(file.artist.into_iter().map(Tag::artist));
                                        vec.extend(file.album.map(Tag::album));
                                        vec.extend(file.genre.into_iter().map(Tag::genre));
                                        vec.extend(file.title.map(Tag::title));
                                        vec.extend(
                                            file.track.map(|track| Tag::track(track.to_string())),
                                        );
                                        vec.extend(
                                            file.year.map(|year| Tag::date(year.to_string())),
                                        );
                                        vec
                                    },
                                }),
                            }
                        })
                        .collect());
                }
            }
            Ok(Vec::new())
        }
    }

    async fn queue_current(&mut self) -> Option<QueueEntry> {
        if let Some(position) = self.player.position().await {
            let PlayerGetItemResponse::Item(item) = self
                .kodi_client
                .send_method(PlayerGetItem::all_properties(self.player.id()))
                .await
                .unwrap();
            return Some(QueueEntry {
                song: Song {
                    path: PathBuf::from(&item.file.unwrap()),
                    tags: {
                        let mut vec = Vec::new();
                        vec.extend(item.artist.into_iter().map(Tag::artist));
                        vec.extend(item.album.map(Tag::album));
                        vec.extend(item.title.map(Tag::title));
                        vec
                    },
                    ..Default::default()
                },
                position,
                id: usize_to_bstring(item.id.unwrap()),
            });
        } else {
            None
        }
    }

    async fn queue_list(&mut self, range: Option<RangeInclusive<usize>>) -> Vec<QueueEntry> {
        let mut items = Vec::new();
        let playlist_items = self.player.playlist_items().await;
        let (start, range) = if let Some(range) = range {
            (*range.start(), playlist_items.get(range).unwrap_or(&[][..]))
        } else if playlist_items.is_empty() {
            (0, &[][..])
        } else {
            (0, &playlist_items[..])
        };
        items.extend(range.iter().enumerate().map(|(idx, item)| QueueEntry {
            song: Song {
                path: PathBuf::from(item.file.as_ref().unwrap()),
                tags: {
                    let mut vec = Vec::new();
                    vec.extend(item.artist.clone().into_iter().map(Tag::artist));
                    vec.extend(item.album.clone().map(Tag::album));
                    vec.extend(item.title.clone().map(Tag::title));
                    vec
                },
                ..Default::default()
            },
            position: idx + start,
            // TODO: properly files without library id
            id: usize_to_bstring(item.id.unwrap_or(usize::MAX)),
        }));
        items
    }

    async fn queue_get(&mut self, id: &BStr) -> Option<QueueEntry> {
        for item in self.queue_list(None).await {
            if item.id == id {
                return Some(item);
            }
        }
        None
    }

    async fn queue_add_file(
        &mut self,
        url: &Url,
        position: Option<usize>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        use kodi_jsonrpc_client::types::playlist::*;

        let playlist_id = self.player.playlist().await.unwrap();

        let path = url.to_file_path().unwrap();
        let path = self
            .path_remap(path.strip_prefix("/").unwrap())
            .await
            .unwrap();

        let FilesGetFileDetailsResponse::FileDetails(details) = self
            .kodi_client
            .send_method(FilesGetFileDetails::all_properties(
                path.to_str().unwrap().to_owned(),
                kodi_jsonrpc_client::types::files::Media::Music,
            ))
            .await?;

        let item = Item::File {
            path: path.to_str().unwrap().to_owned(),
        };

        if let Some(position) = position {
            self.kodi_client
                .send_method(PlaylistInsert {
                    id: playlist_id,
                    position,
                    item: vec![item],
                })
                .await
                .unwrap();
        } else {
            self.kodi_client
                .send_method(PlaylistAdd {
                    id: playlist_id,
                    item: vec![item],
                })
                .await
                .unwrap();
        }
        Ok(details.id.unwrap())
    }

    async fn queue_add(
        &mut self,
        url: &Url,
        position: Option<usize>,
    ) -> Result<Option<usize>, Box<dyn std::error::Error + Send + Sync>> {
        use kodi_jsonrpc_client::types::list::item::FileType;
        use kodi_jsonrpc_client::types::playlist::*;

        let playlist_id = self.player.playlist().await.unwrap();

        let path = url.to_file_path().unwrap();
        let path = self
            .path_remap(path.strip_prefix("/").unwrap())
            .await
            .unwrap();

        let filetype = match self
            .kodi_client
            .send_method(FilesGetFileDetails::all_properties(
                path.to_str().unwrap().to_owned(),
                kodi_jsonrpc_client::types::files::Media::Files,
            ))
            .await
        {
            Ok(_) => FileType::File,
            Err(_) => {
                self.kodi_client
                    .send_method(FilesGetDirectory::all_properties(
                        path.to_str().unwrap().to_owned(),
                        kodi_jsonrpc_client::types::files::Media::Files,
                    ))
                    .await?;
                FileType::Directory
            }
        };

        let item = match filetype {
            FileType::File => Item::File {
                path: path.to_str().unwrap().to_owned(),
            },
            KodiFileType::Directory => Item::Directory {
                path: path.to_str().unwrap().to_owned(),
                media: kodi_jsonrpc_client::types::files::Media::Music,
                recursive: true,
            },
        };

        if let Some(position) = position {
            self.kodi_client
                .send_method(PlaylistInsert {
                    id: playlist_id,
                    position,
                    item: vec![item],
                })
                .await
                .unwrap();
        } else {
            self.kodi_client
                .send_method(PlaylistAdd {
                    id: playlist_id,
                    item: vec![item],
                })
                .await
                .unwrap();
        }
        Ok(None)
    }

    async fn queue_swap(
        &mut self,
        song1: QueueSong,
        song2: QueueSong,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(id) = self.player.playlist().await {
            let position1 = match song1 {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid).await {
                    Some(songpos) => songpos,
                    None => return Ok(()),
                },
                QueueSong::Pos(songpos) => songpos,
            };
            let position2 = match song2 {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid).await {
                    Some(songpos) => songpos,
                    None => return Ok(()),
                },
                QueueSong::Pos(songpos) => songpos,
            };
            self.kodi_client
                .send_method(PlaylistSwap {
                    id,
                    position1,
                    position2,
                })
                .await?;
        }
        Ok(())
    }

    async fn queue_delete(
        &mut self,
        range: RangeInclusive<usize>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(id) = self.player.playlist().await {
            for position in range {
                self.kodi_client
                    .send_method(PlaylistRemove { id, position })
                    .await?;
            }
        }
        Ok(())
    }

    async fn queue_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(id) = self.player.playlist().await {
            self.kodi_client.send_method(PlaylistClear { id }).await?;
        }
        Ok(())
    }

    async fn previous(&mut self) {
        use kodi_jsonrpc_client::types::player::*;

        self.kodi_client
            .send_method(PlayerGoTo {
                id: self.player.id(),
                to: GoTo::Relative(RelativePosition::Previous),
            })
            .await
            .unwrap();
    }

    async fn play(&mut self, song: Option<QueueSong>) {
        let playlist_id = match self.player.playlist().await {
            Some(id) => id,
            None => return,
        };

        match song {
            Some(QueueSong::Id(songid)) => {
                if let Some(songpos) = self.song_id_to_pos(songid).await {
                    self.kodi_client
                        .send_method(PlayerOpen {
                            item: PlayerOpenItem::PlaylistAt {
                                id: playlist_id as usize,
                                position: songpos,
                            },
                            options: Default::default(),
                        })
                        .await
                        .unwrap();
                }
            }
            Some(QueueSong::Pos(songpos)) => {
                self.kodi_client
                    .send_method(PlayerOpen {
                        item: PlayerOpenItem::PlaylistAt {
                            id: playlist_id as usize,
                            position: songpos,
                        },
                        options: Default::default(),
                    })
                    .await
                    .unwrap();
            }
            None => {
                self.kodi_client
                    .send_method(PlayerOpen {
                        item: PlayerOpenItem::PlaylistAt {
                            id: playlist_id as usize,
                            position: 0,
                        },
                        options: Default::default(),
                    })
                    .await
                    .unwrap();
            }
        }
    }

    async fn next(&mut self) {
        use kodi_jsonrpc_client::types::player::*;

        self.kodi_client
            .send_method(PlayerGoTo {
                id: self.player.id(),
                to: GoTo::Relative(RelativePosition::Next),
            })
            .await
            .unwrap();
    }

    async fn stop(&mut self) {
        self.kodi_client
            .send_method(PlayerStop::new(self.player.id()))
            .await
            .unwrap();
    }

    async fn pause(&mut self, pause: Option<bool>) {
        let play = match pause {
            None => kodi_jsonrpc_client::types::global::Toggle::Toggle,
            Some(val) => kodi_jsonrpc_client::types::global::Toggle::Value(!val),
        };
        self.kodi_client
            .send_method(PlayerPlayPause {
                id: self.player.id(),
                play,
            })
            .await
            .unwrap();
    }

    async fn random(
        &mut self,
        state: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.kodi_client
            .send_method(PlayerSetShuffle {
                id: self.player.id(),
                shuffle: kodi_jsonrpc_client::types::global::Toggle::Value(state),
            })
            .await?;
        Ok(())
    }

    async fn seek(
        &mut self,
        song: QueueSong,
        time: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(playlist_id) = self.player.playlist().await {
            let position = match song {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid).await {
                    Some(songpos) => songpos,
                    None => return Ok(()),
                },
                QueueSong::Pos(songpos) => songpos,
            };
            if self.player.position().await != Some(position) {
                self.kodi_client
                    .send_method(PlayerOpen {
                        item: PlayerOpenItem::PlaylistAt {
                            id: playlist_id as usize,
                            position,
                        },
                        options: Default::default(),
                    })
                    .await?;
            }
            self.kodi_client
                .send_method(PlayerSeek {
                    id: self.player.id(),
                    value: PlayerSeekMode::Time(time.into()),
                })
                .await?;
            self.player.event_new(MPDSubsystem::Player);
        }
        Ok(())
    }

    async fn seek_current(
        &mut self,
        time: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.kodi_client
            .send_method(PlayerSeek {
                id: self.player.id(),
                value: PlayerSeekMode::Time(time.into()),
            })
            .await?;
        self.player.event_new(MPDSubsystem::Player);
        Ok(())
    }

    async fn volume_get(&mut self) -> usize {
        self.player.volume().await.unwrap().into()
    }

    async fn volume_set(&mut self, level: usize) {
        self.kodi_client
            .send_method(ApplicationSetVolume { volume: level })
            .await
            .unwrap();
    }

    async fn library_update(
        &mut self,
        uri: Option<&Url>,
        _: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let directory = if let Some(uri) = uri {
            let path = uri.to_file_path().unwrap();
            let path = self
                .path_remap(path.strip_prefix("/").unwrap())
                .await
                .unwrap();
            Some(path.to_str().unwrap().to_string())
        } else {
            None
        };
        self.kodi_client
            .send_method(AudioLibraryScan {
                directory,
                showdialogs: true,
            })
            .await?;
        Ok(())
    }

    async fn library_list(
        &mut self,
        tag: TagType,
        filters: &[TagFilter],
        groups: &[TagType],
    ) -> Result<Vec<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        if !groups.is_empty() {
            return Err("groups are not supported".to_string().into());
        }
        match tag {
            TagType::Album => {
                let mut albums = self
                    .kodi_client
                    .send_method(AudioLibraryGetAlbums::all_properties())
                    .await?
                    .albums;
                for filter in filters {
                    match filter.tag {
                        TagType::Album => {
                            albums = albums
                                .into_iter()
                                .filter(|details| details.label == filter.value)
                                .collect();
                        }
                        TagType::Artist => {
                            albums = albums
                                .into_iter()
                                .filter(|details| details.artist.contains(&filter.value))
                                .collect();
                        }
                        _ => {
                            return Err(
                                format!("filtering on tag '{}' not supported", filter.tag).into()
                            )
                        }
                    }
                }

                Ok(albums
                    .into_iter()
                    .map(|album| Tag {
                        kind: TagType::Album,
                        value: album.label,
                    })
                    .collect())
            }
            TagType::Artist | TagType::AlbumArtist => {
                let mut artists = self
                    .kodi_client
                    .send_method(AudioLibraryGetArtists::all_properties())
                    .await?
                    .artists;
                if tag == TagType::AlbumArtist {
                    artists = artists
                        .into_iter()
                        .filter(|details| details.isalbumartist)
                        .collect();
                }
                for filter in filters {
                    match filter.tag {
                        TagType::Album => {
                            artists = artists
                                .into_iter()
                                .filter(|details| details.label == filter.value)
                                .collect();
                        }
                        TagType::Artist => {
                            artists = artists
                                .into_iter()
                                .filter(|details| details.artist.contains(&filter.value))
                                .collect();
                        }
                        _ => {
                            return Err(
                                format!("filtering on tag '{}' not supported", filter.tag).into()
                            )
                        }
                    }
                }
                Ok(artists
                    .into_iter()
                    .map(|artist| Tag {
                        kind: TagType::Artist,
                        value: artist.label,
                    })
                    .collect())
            }
            _ => Err(format!("unsupported tag: {}", tag).into()),
        }
    }

    async fn library_find(
        &mut self,
        filters: &[TagFilter],
    ) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        use kodi_jsonrpc_client::types::list::filter::fields::Songs as SongsFields;
        use kodi_jsonrpc_client::types::list::filter::rule::Songs as SongsRule;
        use kodi_jsonrpc_client::types::list::filter::{Operators, Songs as SongsFiler};

        let mut filter: Option<SongsFiler> = None;
        for tag_filter in filters {
            let item = match tag_filter.tag {
                TagType::Album => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Album,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::AlbumArtist => SongsFiler::Rule(SongsRule {
                    field: SongsFields::AlbumArtist,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Artist => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Artist,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Comment => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Comment,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Date => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Year,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Genre => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Genre,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Title => SongsFiler::Rule(SongsRule {
                    field: SongsFields::Title,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                TagType::Track => SongsFiler::Rule(SongsRule {
                    field: SongsFields::TrackNumber,
                    operator: Operators::Is,
                    value: tag_filter.value.clone().into(),
                }),
                _ => return Err("Unsupported filter".into()),
            };
            if let Some(filter) = &mut filter {
                filter.and(item);
            } else {
                filter = Some(item);
            }
        }
        let mut method = AudioLibraryGetSongs::all_properties();
        if let Some(filter) = filter {
            method.filter = Some(filter.into());
        }
        let mut songs = Vec::new();
        for song in self.kodi_client.send_method(method).await?.songs {
            songs.push(Song {
                path: song.file.unwrap().into(),
                last_modified: None,
                format: None,
                duration: song.duration,
                tags: {
                    let mut vec = Vec::new();
                    vec.extend(song.artist.into_iter().map(Tag::artist));
                    vec.extend(song.album.map(Tag::album));
                    vec.extend(song.genre.into_iter().map(Tag::genre));
                    vec.extend(song.title.map(Tag::title));
                    vec.extend(song.track.map(|track| Tag::track(track.to_string())));
                    vec.extend(song.year.map(|year| Tag::date(year.to_string())));
                    vec
                },
            });
        }
        Ok(songs)
    }

    async fn idle(
        &mut self,
        wanted: EnumSet<MPDSubsystem>,
    ) -> Result<EnumSet<MPDSubsystem>, Box<dyn std::error::Error + Send + Sync>> {
        self.subsystem_notifier.changed().await?;
        let version = *self.subsystem_notifier.borrow();
        if version > self.subsystem_version {
            self.subsystem_version = version;
            Ok(self.events(wanted))
        } else {
            Ok(EnumSet::empty())
        }
    }

    async fn tags_enable(
        &mut self,
        tags: EnumSet<TagType>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tags.insert_all(tags);
        Ok(())
    }

    async fn tags_disable(
        &mut self,
        tags: EnumSet<TagType>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tags.remove_all(tags);
        Ok(())
    }

    async fn tags_get(
        &mut self,
    ) -> Result<EnumSet<TagType>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.tags)
    }
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    /// Sets kodi JSON-RPC endpoint
    #[clap(short, long, default_value = "http://127.0.0.1:8080/jsonrpc")]
    kodi: reqwest::Url,

    /// Sets listening socket address
    #[clap(short, long, default_value = "127.0.0.1:6600")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();

    let listener = TcpListener::bind(opts.listen).await?;

    let kodi_client = KodiClient::new(
        reqwest::Client::builder().build().unwrap(),
        opts.kodi.clone(),
    );

    let (tx, rx) = watch::channel(0);

    let player = Arc::new(player::KodiPlayer::new(kodi_client, tx));

    let main_player = player.clone();

    tokio::spawn(async move {
        loop {
            main_player.refresh().await;
            sleep(Duration::from_millis(1000)).await;
        }
    });

    loop {
        let (socket, _) = listener.accept().await?;

        let kodi_url = opts.kodi.clone();

        let player = player.clone();

        let rx = rx.clone();

        tokio::spawn(async move {
            let kodi_client =
                KodiClient::new(reqwest::Client::builder().build().unwrap(), kodi_url);

            let mut server = Server::new(
                BufReader::new(socket),
                KodiProxyCommandHandler::new(kodi_client, player, rx),
            )
            .await
            .unwrap();

            loop {
                match server.poll().await {
                    Ok(true) => {}
                    Ok(false) => {
                        event!(Level::DEBUG, "client has exited");
                        return;
                    }
                    Err(err) => {
                        event!(Level::ERROR, "failed to read command; err = {:?}", err);
                        return;
                    }
                };
            }
        });
    }
}
