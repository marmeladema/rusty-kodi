use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::KodiClient;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{event, Level};

pub(crate) struct KodiPlayer {
    kodi_client: KodiClient,
    id: AtomicU8,
    properties: RwLock<kodi_jsonrpc_client::types::player::property::Value>,
    playlist_items: RwLock<Arc<Box<[kodi_jsonrpc_client::types::list::item::All]>>>,
    playlist_version: AtomicU32,
}

impl KodiPlayer {
    pub fn new(kodi_client: KodiClient) -> Self {
        Self {
            kodi_client,
            id: AtomicU8::new(0),
            properties: RwLock::new(Default::default()),
            playlist_items: RwLock::new(Arc::new(Vec::new().into_boxed_slice())),
            playlist_version: AtomicU32::new(0),
        }
    }

    pub async fn refresh(&self) {
        use kodi_jsonrpc_client::types::player::Type as PlayerType;

        let mut ids = &mut [0u8, 1u8, 2u8][..];

        let mut current = self.id();
        assert!(current <= 2);

        while !ids.is_empty() {
            let player_id = ids[current as usize];
            match self
                .kodi_client
                .send_method(PlayerGetProperties::all(player_id))
                .await
            {
                Ok(props) => {
                    if props.kind == Some(PlayerType::Audio) {
                        self.id.store(current, Ordering::Relaxed);
                        *self.properties.write().unwrap() = props;
                        self.refresh_playlist().await;
                        break;
                    }
                }
                Err(err) => event!(
                    Level::ERROR,
                    "Count not retrieve properties of player {}: {}",
                    player_id,
                    err
                ),
            }
            // put current player id at the end
            ids.swap(current.into(), ids.len() - 1);
            // remove last element from the list
            let len = ids.len();
            ids = &mut ids[..(len - 1)];
            // use first id in the list as next player id to try
            current = 0;
        }
    }

    async fn refresh_playlist(&self) {
        if let Some(playlist_id) = self.playlist() {
            match self
                .kodi_client
                .send_method(PlaylistGetItems::all_properties(playlist_id))
                .await
            {
                Ok(PlaylistGetItemsResponse { items, .. }) => {
                    if &***self.playlist_items.read().unwrap() != items {
                        *self.playlist_items.write().unwrap() = Arc::new(items.into_boxed_slice());
                        self.playlist_version.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(err) => event!(
                    Level::ERROR,
                    "Could not retrieve items of playlist {}: {}",
                    playlist_id,
                    err
                ),
            }
        }
    }

    pub fn id(&self) -> u8 {
        self.id.load(Ordering::Relaxed)
    }

    pub fn playlist(&self) -> Option<u8> {
        self.properties.read().unwrap().playlistid
    }

    pub fn position(&self) -> Option<usize> {
        self.properties.read().unwrap().position
    }

    pub fn speed(&self) -> Option<i64> {
        self.properties.read().unwrap().speed
    }

    pub fn shuffled(&self) -> Option<bool> {
        self.properties.read().unwrap().shuffled
    }

    pub fn time(&self) -> Option<Duration> {
        self.properties.read().unwrap().time.map(Duration::from)
    }

    pub fn totaltime(&self) -> Option<Duration> {
        self.properties
            .read()
            .unwrap()
            .totaltime
            .map(Duration::from)
    }

    pub fn playlist_items(&self) -> Arc<Box<[kodi_jsonrpc_client::types::list::item::All]>> {
        self.playlist_items.read().unwrap().clone()
    }

    pub fn playlist_version(&self) -> u32 {
        self.playlist_version.load(Ordering::Relaxed)
    }
}
