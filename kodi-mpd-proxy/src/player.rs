use kodi_jsonrpc_client::methods::*;
use kodi_jsonrpc_client::KodiClient;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use tracing::{event, Level};

pub(crate) struct KodiPlayer {
    kodi_client: KodiClient,
    id: AtomicU8,
    properties: RwLock<kodi_jsonrpc_client::types::player::property::Value>,
}

impl KodiPlayer {
    pub fn new(kodi_client: KodiClient) -> Self {
        Self {
            kodi_client,
            id: AtomicU8::new(0),
            properties: RwLock::new(Default::default()),
        }
    }

    pub async fn refresh(&self) {
        use kodi_jsonrpc_client::types::player::Type as PlayerType;

        let mut ids = &mut [0u8, 1u8, 2u8][..];

        let mut current = self.id();
        assert!(current <= 2);

        while !ids.is_empty() {
            match self
                .kodi_client
                .send_method(PlayerGetProperties::all(ids[current as usize]))
                .await
            {
                Ok(props) => {
                    if props.kind == Some(PlayerType::Audio) {
                        self.id.store(current, Ordering::Relaxed);
                        *self.properties.write().unwrap() = props;
                        break;
                    }
                }
                Err(err) => event!(Level::ERROR, "{}", err),
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
}
