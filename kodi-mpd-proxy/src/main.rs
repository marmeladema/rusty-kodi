use async_trait::async_trait;
use bstr::{BStr, BString};
use clap::Clap;
use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::types::list::item::FileType as KodiFileType;
use kodi_jsonrpc_client::KodiClient;
use mpd_server_protocol::{
    CommandHandler, DirEntry, File, MPDState, MPDStatus, QueueEntry, Server, Url,
};
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpListener;

struct KodiProxyCommandHandler {
    kodi_client: KodiClient,
}

impl KodiProxyCommandHandler {
    fn new(kodi_client: KodiClient) -> Self {
        Self { kodi_client }
    }

    async fn active_player(&self) -> Option<u8> {
        for player in self
            .kodi_client
            .send_method(PlayerGetActivePlayers {})
            .await
            .unwrap()
        {
            if player.kind == kodi_jsonrpc_client::types::player::Type::Audio {
                return Some(player.id);
            }
        }
        None
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
        if let Some(id) = self.active_player().await {
            let player_props = self
                .kodi_client
                .send_method(PlayerGetProperties::all(id))
                .await
                .unwrap();
            let PlayerGetItemResponse::Item(item) = self
                .kodi_client
                .send_method(PlayerGetItem::all_properties(id))
                .await
                .unwrap();
            let playlist_id = player_props.playlistid.unwrap();
            let playlist_props = self
                .kodi_client
                .send_method(PlaylistGetProperties::all(playlist_id))
                .await
                .unwrap();
            if let Some(speed) = player_props.speed {
                if speed == 0 {
                    status.state = MPDState::Pause;
                } else {
                    status.state = MPDState::Play;
                }
            }
            status.random = player_props.shuffled;
            status.song = player_props.position;
            status.songid = item.id;
            status.elapsed = player_props.time.map(Duration::from);
            status.duration = player_props.totaltime.map(Duration::from);
            status.playlist = Some(playlist_id.into());
            status.playlistlength = playlist_props.size;
        }
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
        if let Some(id) = self.active_player().await {
            let props = self
                .kodi_client
                .send_method(PlayerGetProperties::all(id))
                .await
                .unwrap();
            let PlayerGetItemResponse::Item(item) = self
                .kodi_client
                .send_method(PlayerGetItem::all_properties(id))
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
                position: props.position.unwrap(),
                id: usize_to_bstring(item.id.unwrap()),
            });
        }
        None
    }

    async fn queue_list(&mut self, range: Option<RangeInclusive<usize>>) -> Vec<QueueEntry> {
        let mut items = Vec::new();
        if let Some(id) = self.active_player().await {
            let props = self
                .kodi_client
                .send_method(PlayerGetProperties::all(id))
                .await
                .unwrap();
            if let Some(playlist_id) = props.playlistid {
                let mut method = PlaylistGetItems::all_properties(playlist_id);
                method.limits = range.map(|range| range.into());
                let response = self.kodi_client.send_method(method).await.unwrap();
                let limits = response.limits;
                items.extend(response.items.into_iter().enumerate().map(|(idx, item)| {
                    QueueEntry {
                        path: PathBuf::from(&item.file.unwrap()),
                        file: File {
                            artist: item.artist,
                            album: item.album,
                            title: item.title,
                            ..Default::default()
                        },
                        position: idx + limits.start,
                        id: usize_to_bstring(item.id.unwrap()),
                    }
                }));
            }
        }
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

    async fn queue_add(&mut self, path: &Path, position: Option<usize>) -> Option<usize> {
        use kodi_jsonrpc_client::types::list::item::FileType;
        use kodi_jsonrpc_client::types::playlist::*;

        let playlist_id = 2;

        let FilesGetFileDetailsResponse::FileDetails(details) = self
            .kodi_client
            .send_method(FilesGetFileDetails::all_properties(
                path.to_str().unwrap().to_owned(),
                kodi_jsonrpc_client::types::files::Media::Files,
            ))
            .await
            .unwrap();

        let item = match details.filetype {
            FileType::File => Item::File {
                path: path.to_str().unwrap().to_owned(),
            },
            KodiFileType::Directory => Item::Directory {
                path: path.to_str().unwrap().to_owned(),
                media: kodi_jsonrpc_client::types::files::Media::Files,
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
        None
    }

    async fn previous(&mut self) {
        use kodi_jsonrpc_client::types::player::*;

        if let Some(id) = self.active_player().await {
            self.kodi_client
                .send_method(PlayerGoTo {
                    id,
                    to: GoTo::Relative(RelativePosition::Previous),
                })
                .await
                .unwrap();
        }
    }

    async fn play(&mut self, pos: usize) {
        use kodi_jsonrpc_client::types::player::*;

        if let Some(id) = self.active_player().await {
            self.kodi_client
                .send_method(PlayerGoTo {
                    id,
                    to: GoTo::Absolute(pos),
                })
                .await
                .unwrap();
        }
    }

    async fn next(&mut self) {
        use kodi_jsonrpc_client::types::player::*;

        if let Some(id) = self.active_player().await {
            self.kodi_client
                .send_method(PlayerGoTo {
                    id,
                    to: GoTo::Relative(RelativePosition::Next),
                })
                .await
                .unwrap();
        }
    }

    async fn stop(&mut self) {
        if let Some(id) = self.active_player().await {
            self.kodi_client
                .send_method(PlayerStop::new(id))
                .await
                .unwrap();
        }
    }

    async fn pause(&mut self, pause: Option<bool>) {
        if let Some(id) = self.active_player().await {
            let play = match pause {
                None => kodi_jsonrpc_client::types::global::Toggle::Toggle,
                Some(val) => kodi_jsonrpc_client::types::global::Toggle::Value(!val),
            };
            self.kodi_client
                .send_method(PlayerPlayPause { id, play })
                .await
                .unwrap();
        }
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

    loop {
        let (socket, _) = listener.accept().await?;

        let kodi_url = opts.kodi.clone();

        tokio::spawn(async move {
            let kodi_client =
                KodiClient::new(reqwest::Client::builder().build().unwrap(), kodi_url);

            let mut server = Server::new(
                BufReader::new(socket),
                KodiProxyCommandHandler::new(kodi_client),
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

#[test]
fn test_uri() {
    let base = Url::parse("file://").unwrap();
    let options = Url::options().base_url(Some(&base));
    let url = options.parse("a//b");
    println!("url: {:?}", url);
    println!("url2: {:?}", Url::from_file_path("q//b"));
    assert!(false);
}
