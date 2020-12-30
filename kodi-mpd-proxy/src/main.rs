use async_trait::async_trait;
use bstr::{BStr, BString};
use clap::Clap;
use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::types::list::item::FileType as KodiFileType;
use kodi_jsonrpc_client::KodiClient;
use mpd_server_protocol::{
    CommandHandler, DirEntry, File, MPDState, MPDStatus, QueueEntry, QueueSong, Server, Url,
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
use tokio::time::delay_for;

mod player;

struct KodiProxyCommandHandler {
    kodi_client: KodiClient,
    player: Arc<player::KodiPlayer>,
}

impl KodiProxyCommandHandler {
    fn new(kodi_client: KodiClient, player: Arc<player::KodiPlayer>) -> Self {
        Self {
            kodi_client,
            player,
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

    fn song_id_to_pos(&self, songid: usize) -> Option<usize> {
        for (pos, item) in self.player.playlist_items().iter().enumerate().rev() {
            if item.id == Some(songid) {
                return Some(pos);
            }
        }
        None
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
        let app_props = self
            .kodi_client
            .send_method(ApplicationGetProperties {
                properties: kodi_jsonrpc_client::types::application::property::Name::Volume.into(),
            })
            .await
            .unwrap();

        let mut status = MPDStatus {
            volume: app_props.volume,
            ..Default::default()
        };
        let PlayerGetItemResponse::Item(item) = self
            .kodi_client
            .send_method(PlayerGetItem::all_properties(self.player.id()))
            .await
            .unwrap();
        if let Some(playlist_id) = self.player.playlist() {
            let playlist_props = self
                .kodi_client
                .send_method(PlaylistGetProperties::all(playlist_id))
                .await
                .unwrap();
            status.playlistlength = playlist_props.size;
        }
        if self.player.position().is_some() {
            if let Some(speed) = self.player.speed() {
                if speed == 0 {
                    status.state = MPDState::Pause;
                } else {
                    status.state = MPDState::Play;
                }
            }
        }
        status.random = self.player.shuffled();
        status.song = self.player.position();
        status.songid = item.id;
        status.elapsed = self.player.time();
        status.duration = self.player.totaltime();
        status.playlist = Some(self.player.playlist_version());
        status
    }

    async fn list_directory(
        &mut self,
        url: Option<&Url>,
    ) -> Result<Vec<DirEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self
            .kodi_client
            .send_method(AudioLibraryGetSources::default())
            .await?;
        let path = url
            .map(|url| url.to_file_path().unwrap())
            .unwrap_or(PathBuf::new());
        if path == Path::new("/") || path == Path::new("") {
            Ok(resp
                .sources
                .into_iter()
                .map(|source| DirEntry {
                    path: PathBuf::from(source.label),
                    last_modified: None,
                    file: None,
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
                            let file = match file.filetype {
                                KodiFileType::Directory => None,
                                KodiFileType::File => Some(File {
                                    duration: file.duration,
                                    artist: file.artist,
                                    album: file.album,
                                    genre: file.genre,
                                    title: file.title,
                                    track: file.track,
                                    year: file.year,
                                }),
                            };
                            DirEntry {
                                path,
                                last_modified: None,
                                file,
                            }
                        })
                        .collect());
                }
            }
            Ok(Vec::new())
        }
    }

    async fn queue_current(&mut self) -> Option<QueueEntry> {
        if let Some(position) = self.player.position() {
            let PlayerGetItemResponse::Item(item) = self
                .kodi_client
                .send_method(PlayerGetItem::all_properties(self.player.id()))
                .await
                .unwrap();
            return Some(QueueEntry {
                path: PathBuf::from(&item.file.unwrap()),
                file: File {
                    artist: item.artist,
                    album: item.album,
                    title: item.title,
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
        let playlist_items = self.player.playlist_items();
        let (start, range) = if let Some(range) = range {
            (*range.start(), playlist_items.get(range).unwrap_or(&[][..]))
        } else if playlist_items.is_empty() {
            (0, &[][..])
        } else {
            (0, &playlist_items[..])
        };
        items.extend(range.iter().enumerate().map(|(idx, item)| QueueEntry {
            path: PathBuf::from(item.file.as_ref().unwrap()),
            file: File {
                artist: item.artist.clone(),
                album: item.album.as_ref().map(String::from),
                title: item.title.as_ref().map(String::from),
                ..Default::default()
            },
            position: idx + start,
            id: usize_to_bstring(item.id.unwrap()),
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

        let playlist_id = self.player.playlist().unwrap();

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

        let playlist_id = self.player.playlist().unwrap();

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
        if let Some(id) = self.player.playlist() {
            let position1 = match song1 {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid) {
                    Some(songpos) => songpos,
                    None => return Ok(()),
                },
                QueueSong::Pos(songpos) => songpos,
            };
            let position2 = match song2 {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid) {
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
        if let Some(id) = self.player.playlist() {
            for position in range {
                self.kodi_client
                    .send_method(PlaylistRemove { id, position })
                    .await?;
            }
        }
        Ok(())
    }

    async fn queue_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(id) = self.player.playlist() {
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
        let playlist_id = match self.player.playlist() {
            Some(id) => id,
            None => return,
        };

        match song {
            Some(QueueSong::Id(songid)) => {
                if let Some(songpos) = self.song_id_to_pos(songid) {
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
        if let Some(playlist_id) = self.player.playlist() {
            let position = match song {
                QueueSong::Id(songid) => match self.song_id_to_pos(songid) {
                    Some(songpos) => songpos,
                    None => return Ok(()),
                },
                QueueSong::Pos(songpos) => songpos,
            };
            if self.player.position() != Some(position) {
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
        Ok(())
    }

    async fn volume_get(&mut self) -> usize {
        let app_props = self
            .kodi_client
            .send_method(ApplicationGetProperties {
                properties: kodi_jsonrpc_client::types::application::property::Name::Volume.into(),
            })
            .await
            .unwrap();
        app_props.volume.unwrap().into()
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

    let mut listener = TcpListener::bind(opts.listen).await?;

    let kodi_client = KodiClient::new(
        reqwest::Client::builder().build().unwrap(),
        opts.kodi.clone(),
    );

    let player = Arc::new(player::KodiPlayer::new(kodi_client));

    let main_player = player.clone();

    tokio::spawn(async move {
        loop {
            main_player.refresh().await;
            delay_for(Duration::from_millis(1000)).await;
        }
    });

    loop {
        let (socket, _) = listener.accept().await?;

        let kodi_url = opts.kodi.clone();

        let player = player.clone();

        tokio::spawn(async move {
            let kodi_client =
                KodiClient::new(reqwest::Client::builder().build().unwrap(), kodi_url);

            let mut server = Server::new(
                BufReader::new(socket),
                KodiProxyCommandHandler::new(kodi_client, player),
            )
            .await
            .unwrap();

            loop {
                match server.poll().await {
                    Ok(true) => {}
                    Ok(false) => {
                        println!("client has exited");
                        return;
                    }
                    Err(err) => {
                        eprintln!("failed to read command; err = {:?}", err);
                        return;
                    }
                };
            }
        });
    }
}
