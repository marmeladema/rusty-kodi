use clap::Clap;
use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::KodiClient;
use reqwest::header::{HeaderMap, HeaderValue, CONNECTION};
use reqwest::Url;
use tracing::{event, Level};

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    /// Sets kodi JSON-RPC endpoint
    #[clap(short, long, default_value = "http://127.0.0.1:8080/jsonrpc")]
    kodi: Url,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();

    let mut headers = HeaderMap::new();
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .http1_title_case_headers()
        .build()?;
    let client = KodiClient::new(client, opts.kodi);

    let resp = client.send_method(JSONRPCVersion {}).await?;
    event!(Level::INFO, "JSONRPC.Version:\n{:#?}", resp);

    let resp = client
        .send_method(AudioLibraryGetSources::default())
        .await?;
    event!(Level::INFO, "AudioLibrary.GetSources:\n{:#?}", resp);

    for source in resp.sources {
        let resp = client
            .send_method(FilesGetDirectory {
                directory: source.file.to_owned(),
                ..Default::default()
            })
            .await?;
        event!(
            Level::INFO,
            "Files.GetDirectory({:#?}):\n{:#?}",
            source.file,
            resp
        );
    }

    let resp = client.send_method(ApplicationGetProperties::all()).await?;
    event!(Level::INFO, "Application.GetProperties:\n{:#?}", resp);

    let resp = client.send_method(PlayerGetActivePlayers {}).await?;
    event!(Level::INFO, "Player.GetActivePlayers:\n{:#?}", resp);

    for player in resp {
        let resp = client
            .send_method(PlayerGetProperties::all(player.id))
            .await?;
        event!(
            Level::INFO,
            "Player.GetProperties({:#?}):\n{:#?}",
            player.id,
            resp
        );

        let resp = client
            .send_method(PlayerGetItem::all_properties(player.id))
            .await?;
        event!(
            Level::INFO,
            "Player.GetItem({:#?}):\n{:#?}",
            player.id,
            resp
        );
    }

    let resp = client
        .send_method(AudioLibraryGetArtists::all_properties())
        .await?;
    event!(Level::INFO, "AudioLibrary.GetArtists:\n{:#?}", resp.artists);

    let resp = client
        .send_method(AudioLibraryGetAlbums::all_properties())
        .await?;
    event!(Level::INFO, "AudioLibrary.GetAlbums:\n{:#?}", resp.albums);

    let resp = client
        .send_method(AudioLibraryGetSongs::all_properties())
        .await?;
    event!(Level::INFO, "AudioLibrary.GetSongs:\n{:#?}", resp.songs);

    Ok(())
}
