pub mod application {
    pub mod property {
        use enumset::EnumSetType;

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[enumset(serialize_as_list)]
        #[serde(rename_all = "lowercase")]
        pub enum Name {
            Volume,
            Muted,
            Name,
            Version,
            SortTokens,
            Language,
        }

        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(untagged)]
        pub enum Revision {
            Str(String),
            Int(i64),
        }

        #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum Tag {
            PreAlpha,
            Alpha,
            Beta,
            ReleaseCandidate,
            Stable,
        }

        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Version {
            pub major: usize,
            pub minor: usize,
            pub revision: Option<Revision>,
            pub tag: Tag,
            pub tagversion: Option<String>,
        }

        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Value {
            pub language: Option<String>,
            pub muted: Option<bool>,
            pub name: Option<String>,
            pub version: Option<Version>,
            pub volume: Option<u8>,
        }
    }
}

pub mod audio {
    pub mod details {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Artist {
            pub label: String,
            pub fanart: Option<String>,
            pub thumbnail: Option<String>,
            pub art: Option<crate::types::media::Artwork>,
            pub dateadded: Option<String>,
            #[serde(default)]
            pub genre: Vec<String>,
            pub artist: String,
            pub artistid: usize,
            pub born: Option<String>,
            #[serde(default)]
            pub compilationartist: bool,
            pub description: Option<String>,
            pub died: Option<String>,
            pub disambiguation: Option<String>,
            pub disbanded: Option<String>,
            pub formed: Option<String>,
            pub gender: Option<String>,
            #[serde(default)]
            pub instrument: Vec<String>,
            #[serde(default)]
            pub isalbumartist: bool,
            #[serde(default)]
            pub mood: Vec<String>,
            pub musicbrainzartistid: Option<Vec<String>>,
            //pub roles: Option<Audio.Artist.Roles>,
            //pub songgenres: Option<Audio.Details.Genres>,
            pub sortname: Option<String>,
            #[serde(default)]
            pub sourceid: Vec<isize>,
            #[serde(default)]
            pub style: Vec<String>,
            #[serde(rename = "type")]
            pub kind: Option<String>,
            #[serde(default)]
            pub yearsactive: Vec<String>,
        }
    }

    pub mod fields {
        use enumset::EnumSetType;

        /// Requesting the (song)genreid/genre, roleid/role or sourceid fields will result in increased response times
        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[enumset(serialize_as_list)]
        #[serde(rename_all = "lowercase")]
        pub enum Artist {
            Instrument,
            Style,
            Mood,
            Born,
            Formed,
            Description,
            Genre,
            Died,
            Disbanded,
            YearsActive,
            MusicBrainzArtistId,
            FanArt,
            Thumbnail,
            CompilationArtist,
            DateAdded,
            Roles,
            SongGenres,
            IsAlbumArtist,
            SortName,
            Type,
            Gender,
            Disambiguation,
            Art,
            SourceId,
        }
    }
}

pub mod files {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Media {
        Video,
        Music,
        Pictures,
        Files,
        Programs,
    }

    impl Default for Media {
        fn default() -> Self {
            Self::Video
        }
    }
}

pub mod global {
    use std::time::Duration;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum IncrementDecrement {
        Increment,
        Decrement,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Time {
        pub hours: u64,
        pub minutes: u8,
        pub seconds: u8,
        pub milliseconds: i16,
    }

    impl From<Time> for Duration {
        fn from(time: Time) -> Self {
            let minutes: u64 = time.hours * 60u64 + time.minutes as u64;
            let seconds: u64 = minutes * 60u64 + time.seconds as u64;
            let milliseconds: u64 = seconds * 1000u64 + time.milliseconds as u64;
            Duration::from_millis(milliseconds)
        }
    }

    impl From<Duration> for Time {
        fn from(duration: Duration) -> Self {
            let mut total = duration.as_millis() as u64;
            let milliseconds: i16 = (total % 1000) as i16;
            total /= 1000;
            let seconds = (total % 60) as u8;
            total /= 60;
            let minutes = (total % 60) as u8;
            let hours = total / 60;
            Self {
                hours,
                minutes,
                seconds,
                milliseconds,
            }
        }
    }

    fn serialize_toggle<S>(serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("toggle")
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(untagged)]
    pub enum Toggle {
        #[serde(serialize_with = "crate::types::global::serialize_toggle")]
        Toggle,
        Value(bool),
    }

    impl Default for Toggle {
        fn default() -> Self {
            Toggle::Toggle
        }
    }
}

pub mod library {
    pub mod details {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Source {
            pub file: String,
            pub label: String,
            #[serde(default)]
            pub paths: Vec<String>,
            pub sourceid: isize,
        }
    }

    pub mod fields {
        use enumset::EnumSetType;

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[enumset(serialize_as_list)]
        #[serde(rename_all = "lowercase")]
        pub enum Source {
            File,
            Paths,
        }
    }
}

pub mod list {
    pub mod fields {
        use enumset::EnumSetType;

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[enumset(serialize_as_list)]
        #[serde(rename_all = "lowercase")]
        pub enum All {
            Title,
            Artist,
            Albumartist,
            Genre,
            Year,
            Rating,
            Album,
            Track,
            Duration,
            Comment,
            Lyrics,
            Musicbrainztrackid,
            Musicbrainzartistid,
            Musicbrainzalbumid,
            Musicbrainzalbumartistid,
            Playcount,
            Fanart,
            Director,
            Trailer,
            Tagline,
            Plot,
            Plotoutline,
            OriginalTitle,
            LastPlayed,
            Writer,
            Studio,
            Mpaa,
            Cast,
            Country,
            Imdbnumber,
            Premiered,
            ProductionCode,
            Runtime,
            Set,
            Showlink,
            StreamDetails,
            Top250,
            Votes,
            Firstaired,
            Season,
            Episode,
            ShowTitle,
            Thumbnail,
            File,
            Resume,
            ArtistId,
            AlbumId,
            TvShowId,
            Setid,
            Watchedepisodes,
            Disc,
            Tag,
            Art,
            Genreid,
            Displayartist,
            Albumartistid,
            Description,
            Theme,
            Mood,
            Style,
            Albumlabel,
            Sorttitle,
            Episodeguide,
            Uniqueid,
            Dateadded,
            Channel,
            Channeltype,
            Hidden,
            Locked,
            Channelnumber,
            Starttime,
            Endtime,
            Specialsortseason,
            Specialsortepisode,
            Compilation,
            ReleaseType,
            AlbumReleaseType,
            Contributors,
            Displaycomposer,
            Displayconductor,
            Displayorchestra,
            Displaylyricist,
            Userrating,
            Sortartist,
            Musicbrainzreleasegroupid,
            Mediapath,
            Dynpath,
        }

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[enumset(serialize_as_list)]
        #[serde(rename_all = "lowercase")]
        pub enum Files {
            Title,
            Artist,
            AlbumArtist,
            Genre,
            Year,
            Rating,
            Album,
            Track,
            Duration,
            Comment,
            Lyrics,
            MusicBrainzTrackId,
            MusicBrainzArtistId,
            MusicBrainzAlbumId,
            MusicBrainzAlbumArtistId,
            MusicBrainzReleaseGroupId,
            Playcount,
            Fanart,
            Director,
            Trailer,
            Tagline,
            Plot,
            Plotoutline,
            Originaltitle,
            LastPlayed,
            Writer,
            Studio,
            Mpaa,
            Cast,
            Country,
            ImdbNumber,
            Premiered,
            ProductionCode,
            Runtime,
            Set,
            Showlink,
            StreamDetails,
            Top250,
            Votes,
            Firstaired,
            Season,
            Episode,
            ShowTitle,
            Thumbnail,
            File,
            Resume,
            ArtistId,
            AlbumId,
            TvShowId,
            Setid,
            WatchedEpisodes,
            Disc,
            Tag,
            Art,
            Genreid,
            DisplayArtist,
            AlbumArtistId,
            Description,
            Theme,
            Mood,
            Style,
            AlbumLabel,
            SortTitle,
            EpisodeGuide,
            UniqueId,
            DateAdded,
            Size,
            LastModified,
            Mimetype,
            SpecialSortSeason,
            SpecialSortEpisode,
            SortArtist,
        }
    }

    pub mod filter {
        pub mod fields {
            #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
            #[serde(rename_all = "lowercase")]
            pub enum Artists {
                Artist,
                Source,
                Genre,
                Moods,
                Styles,
                Instruments,
                Biography,
                ArtistType,
                Gender,
                Disambiguation,
                Born,
                BandFormed,
                Disbanded,
                Died,
                Role,
                Path,
                Playlist,
                VirtualFolder,
            }

            impl Default for Artists {
                fn default() -> Self {
                    Self::Artist
                }
            }
        }

        pub mod rule {
            #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
            #[serde(rename_all = "lowercase")]
            #[serde(untagged)]
            pub enum Value {
                One(String),
                Many(Vec<String>),
            }

            #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
            pub struct Artists {
                pub operator: crate::types::list::filter::Operators,
                pub value: Value,
                pub field: crate::types::list::filter::fields::Artists,
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum Operators {
            Contains,
            DoesNotContain,
            Is,
            IsNot,
            StartsWith,
            EndsWith,
            GreaterThan,
            LessThan,
            After,
            Before,
            InTheLast,
            NotInTheLast,
            True,
            False,
            Between,
        }

        impl Default for Operators {
            fn default() -> Self {
                Self::Contains
            }
        }

        #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum Artists {
            And(Vec<Artists>),
            Or(Vec<Artists>),
            Rule(crate::types::list::filter::rule::Artists),
        }
    }

    pub mod item {
        #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum ItemKind {
            Unknown,
            Album,
            Movie,
            Episode,
            MusicVideo,
            Song,
            Picture,
            Channel,
        }

        #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub struct All {
            pub label: String,
            pub fanart: Option<String>,
            pub thumbnail: Option<String>,
            pub art: Option<crate::types::media::Artwork>,
            pub playcount: Option<usize>,
            pub title: Option<String>,
            pub dateadded: Option<String>,
            pub file: Option<String>,
            pub lastplayed: Option<String>,
            pub plot: Option<String>,
            pub director: Option<Vec<String>>,
            pub resume: Option<crate::types::video::Resume>,
            pub runtime: Option<usize>,
            // pub streamdetails: Option<Video.Streams>,
            pub genre: Option<Vec<String>>,
            #[serde(default)]
            pub artist: Vec<String>,
            #[serde(default)]
            pub artistid: Vec<isize>,
            pub displayartist: Option<String>,
            pub musicbrainzalbumartistid: Option<Vec<String>>,
            pub rating: Option<f64>,
            pub sortartist: Option<String>,
            pub userrating: Option<usize>,
            pub votes: Option<usize>,
            pub year: Option<usize>,
            pub album: Option<String>,
            #[serde(default)]
            pub albumartist: Vec<String>,
            #[serde(default)]
            pub albumartistid: Vec<isize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub albumid: Option<usize>,
            pub albumlabel: Option<String>,
            // pub albumreleasetype: Option<Audio.Album.ReleaseType>,
            pub cast: Option<crate::types::video::Cast>,
            pub comment: Option<String>,
            pub compilation: Option<bool>,
            // pub contributors: Option<Audio.Contributors>,
            pub country: Option<Vec<String>>,
            pub description: Option<String>,
            pub disc: Option<usize>,
            pub displaycomposer: Option<String>,
            pub displayconductor: Option<String>,
            pub displaylyricist: Option<String>,
            pub displayorchestra: Option<String>,
            pub duration: Option<usize>,
            pub dynpath: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub episode: Option<usize>,
            pub episodeguide: Option<String>,
            pub firstaired: Option<String>,
            pub id: Option<usize>,
            pub imdbnumber: Option<String>,
            pub lyrics: Option<String>,
            pub mediapath: Option<String>,
            pub mood: Option<Vec<String>>,
            pub mpaa: Option<String>,
            pub musicbrainzartistid: Option<Vec<String>>,
            pub musicbrainztrackid: Option<String>,
            pub originaltitle: Option<String>,
            pub plotoutline: Option<String>,
            pub premiered: Option<String>,
            pub productioncode: Option<String>,
            // pub releasetype: Option<Audio.Album.ReleaseType>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub season: Option<usize>,
            pub set: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub setid: Option<usize>,
            pub showlink: Option<Vec<String>>,
            pub showtitle: Option<String>,
            pub sorttitle: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub specialsortepisode: Option<usize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub specialsortseason: Option<usize>,
            pub studio: Option<Vec<String>>,
            pub style: Option<Vec<String>>,
            pub tag: Option<Vec<String>>,
            pub tagline: Option<String>,
            pub theme: Option<Vec<String>>,
            pub top250: Option<usize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub track: Option<usize>,
            pub trailer: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub tvshowid: Option<usize>,
            #[serde(rename = "type")]
            pub kind: Option<ItemKind>,
            pub uniqueid: Option<String>,
            pub watchedepisodes: Option<usize>,
            pub writer: Option<Vec<String>>,
            pub channel: Option<String>,
            pub channelnumber: Option<usize>,
            // pub channeltype: Option<PVR.Channel.Type>,
            pub endtime: Option<String>,
            pub hidden: Option<bool>,
            pub locked: Option<bool>,
            pub starttime: Option<String>,
        }

        #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum FileType {
            File,
            Directory,
        }

        impl Default for FileType {
            fn default() -> Self {
                Self::File
            }
        }

        #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct File {
            pub label: String,
            pub fanart: Option<String>,
            pub thumbnail: Option<String>,
            pub art: Option<crate::types::media::Artwork>,
            pub playcount: Option<usize>,
            pub title: Option<String>,
            pub dateadded: Option<String>,
            pub file: String,
            pub lastplayed: Option<String>,
            pub plot: Option<String>,
            pub director: Option<Vec<String>>,
            pub resume: Option<crate::types::video::Resume>,
            pub runtime: Option<usize>,
            // pub streamdetails: Option<Video.Streams>,
            #[serde(default)]
            pub genre: Vec<String>,
            #[serde(default)]
            pub artist: Vec<String>,
            #[serde(default)]
            pub artistid: Vec<isize>,
            pub displayartist: Option<String>,
            pub musicbrainzalbumartistid: Option<Vec<String>>,
            pub rating: Option<f64>,
            pub sortartist: Option<String>,
            pub userrating: Option<usize>,
            pub votes: Option<usize>,
            pub year: Option<usize>,
            pub album: Option<String>,
            #[serde(default)]
            pub albumartist: Vec<String>,
            #[serde(default)]
            pub albumartistid: Vec<isize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub albumid: Option<usize>,
            pub albumlabel: Option<String>,
            // pub albumreleasetype: Option<Audio.Album.ReleaseType>,
            pub cast: Option<crate::types::video::Cast>,
            pub comment: Option<String>,
            pub compilation: Option<bool>,
            // pub contributors: Option<Audio.Contributors>,
            pub country: Option<Vec<String>>,
            pub description: Option<String>,
            pub disc: Option<usize>,
            pub displaycomposer: Option<String>,
            pub displayconductor: Option<String>,
            pub displaylyricist: Option<String>,
            pub displayorchestra: Option<String>,
            pub duration: Option<usize>,
            pub dynpath: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub episode: Option<usize>,
            pub episodeguide: Option<String>,
            pub firstaired: Option<String>,
            pub id: Option<usize>,
            pub imdbnumber: Option<String>,
            pub lyrics: Option<String>,
            pub mediapath: Option<String>,
            pub mood: Option<Vec<String>>,
            pub mpaa: Option<String>,
            pub musicbrainzartistid: Option<Vec<String>>,
            pub musicbrainztrackid: Option<String>,
            pub originaltitle: Option<String>,
            pub plotoutline: Option<String>,
            pub premiered: Option<String>,
            pub productioncode: Option<String>,
            // pub releasetype: Option<Audio.Album.ReleaseType>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub season: Option<usize>,
            pub set: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub setid: Option<usize>,
            pub showlink: Option<Vec<String>>,
            pub showtitle: Option<String>,
            pub sorttitle: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub specialsortepisode: Option<usize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub specialsortseason: Option<usize>,
            pub studio: Option<Vec<String>>,
            pub style: Option<Vec<String>>,
            pub tag: Option<Vec<String>>,
            pub tagline: Option<String>,
            pub theme: Option<Vec<String>>,
            pub top250: Option<usize>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub track: Option<usize>,
            pub trailer: Option<String>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub tvshowid: Option<usize>,
            #[serde(rename = "type")]
            pub kind: Option<ItemKind>,
            pub uniqueid: Option<String>,
            pub watchedepisodes: Option<usize>,
            pub writer: Option<Vec<String>>,
            #[serde(default)]
            pub filetype: FileType,
            pub lastmodified: Option<String>,
            pub mimetype: Option<String>,
            pub size: Option<usize>,
        }
    }

    use std::ops::RangeInclusive;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Limits {
        pub start: usize,
        pub end: usize,
    }

    impl From<RangeInclusive<usize>> for Limits {
        fn from(range: RangeInclusive<usize>) -> Self {
            Self {
                start: *range.start(),
                end: *range.end(),
            }
        }
    }

    impl From<&RangeInclusive<usize>> for Limits {
        fn from(range: &RangeInclusive<usize>) -> Self {
            Self {
                start: *range.start(),
                end: *range.end(),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct LimitsReturned {
        pub start: usize,
        pub end: usize,
        pub total: usize,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum SortMethod {
        None,
        Label,
        Date,
        Size,
        File,
        Path,
        Drivetype,
        Title,
        Track,
        Time,
        Artist,
        Album,
        Albumtype,
        Genre,
        Country,
        Year,
        Rating,
        Userrating,
        Votes,
        Top250,
        Programcount,
        Playlist,
        Episode,
        Season,
        Totalepisodes,
        Watchedepisodes,
        Tvshowstatus,
        Tvshowtitle,
        Sorttitle,
        Productioncode,
        Mpaa,
        Studio,
        Dateadded,
        Lastplayed,
        Playcount,
        Listeners,
        Bitrate,
        Random,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum SortOrder {
        Ascending,
        Descending,
    }

    impl Default for SortOrder {
        fn default() -> Self {
            Self::Ascending
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Sort {
        pub ignorearticle: bool,
        pub method: SortMethod,
        pub order: SortOrder,
        pub useartistsortname: bool,
    }
}

pub mod media {
    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Artwork {
        pub banner: Option<String>,
        pub fanart: Option<String>,
        pub poster: Option<String>,
        pub thumb: Option<String>,
    }
}

pub mod player {
    pub mod audio {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Stream {
            bitrate: i64,
            channels: u8,
            codec: String,
            index: usize,
            language: String,
            name: String,
        }
    }

    pub mod property {
        use enumset::EnumSetType;

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        #[enumset(serialize_as_list)]
        pub enum Name {
            #[serde(rename = "type")]
            Kind,
            PartyMode,
            Speed,
            Time,
            Percentage,
            TotalTime,
            PlaylistId,
            Position,
            Repeat,
            Shuffled,
            CanSeek,
            CanChangeSpeed,
            CanMove,
            CanZoom,
            CanRotate,
            CanShuffle,
            CanRepeat,
            CurrentAudioStream,
            AudioStreams,
            SubtitleEnabled,
            CurrentSubtitle,
            Subtitles,
            Live,
            CurrentVideoStream,
            VideoStreams,
        }

        fn deserialize_opt_stream<'de, D>(
            deserializer: D,
        ) -> Result<Option<crate::types::player::audio::Stream>, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            use serde::Deserialize;

            #[derive(serde::Deserialize)]
            struct EmptyStream {}

            #[derive(serde::Deserialize)]
            #[serde(untagged)]
            enum OptionStream {
                Stream(crate::types::player::audio::Stream),
                Empty(EmptyStream),
            }

            let opt_stream = Option::<OptionStream>::deserialize(deserializer)?;
            Ok(match opt_stream {
                Some(OptionStream::Empty(_)) | None => None,
                Some(OptionStream::Stream(stream)) => Some(stream),
            })
        }

        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Value {
            pub audiostreams: Option<Vec<crate::types::player::audio::Stream>>,
            pub canchangespeed: Option<bool>,
            pub canmove: Option<bool>,
            pub canrepeat: Option<bool>,
            pub canrotate: Option<bool>,
            pub canseek: Option<bool>,
            pub canshuffle: Option<bool>,
            pub canzoom: Option<bool>,
            #[serde(
                default,
                deserialize_with = "crate::types::player::property::deserialize_opt_stream"
            )]
            pub currentaudiostream: Option<crate::types::player::audio::Stream>,
            pub currensubtitle: Option<crate::types::player::Subtitle>,
            pub currentvideostream: Option<crate::types::player::video::Stream>,
            pub live: Option<bool>,
            pub partymode: Option<bool>,
            // percentage: Option<f32>,
            pub playlistid: Option<u8>,
            #[serde(default, deserialize_with = "crate::deserialize_opt_usize")]
            pub position: Option<usize>,
            #[serde(default)]
            pub repeat: crate::types::player::Repeat,
            #[serde(default, deserialize_with = "crate::deserialize_opt_bool")]
            pub shuffled: Option<bool>,
            pub speed: Option<i64>,
            pub subtitleenabled: Option<bool>,
            pub subtitles: Option<Vec<crate::types::player::Subtitle>>,
            pub time: Option<crate::types::global::Time>,
            pub totaltime: Option<crate::types::global::Time>,
            #[serde(rename = "type")]
            pub kind: Option<crate::types::player::Type>,
            pub videostreams: Option<Vec<crate::types::player::video::Stream>>,
        }
    }

    pub mod video {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Stream {
            codec: String,
            height: usize,
            index: usize,
            language: String,
            name: String,
            width: usize,
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum PlayerSource {
        Internal,
        External,
        Remote,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
    pub struct ActivePlayer {
        #[serde(rename = "playerid")]
        pub id: u8,
        #[serde(rename = "playertype")]
        pub source: PlayerSource,
        #[serde(rename = "type")]
        pub kind: crate::types::player::Type,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum RelativePosition {
        Previous,
        Next,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
    #[serde(untagged)]
    pub enum GoTo {
        Relative(RelativePosition),
        Absolute(usize),
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Repeat {
        Off,
        One,
        All,
    }

    impl Default for Repeat {
        fn default() -> Self {
            Self::Off
        }
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Speed {
        speed: i8,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Subtitle {
        index: usize,
        language: String,
        name: String,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Type {
        Video,
        Audio,
        Picture,
    }

    impl Default for Type {
        fn default() -> Self {
            Self::Video
        }
    }
}

pub mod playlist {
    pub mod property {
        use enumset::EnumSetType;

        #[derive(Debug, EnumSetType, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "lowercase")]
        #[enumset(serialize_as_list)]
        pub enum Name {
            #[serde(rename = "type")]
            Kind,
            Size,
        }

        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct Value {
            #[serde(rename = "type")]
            pub kind: Option<crate::types::playlist::Type>,
            pub size: Option<usize>,
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ItemId {
        MovieId(usize),
        EpisodeId(usize),
        MusicVideoId(usize),
        ArtistId(usize),
        AlbumId(usize),
        SongId(usize),
        GenreId(usize),
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(untagged)]
    pub enum Item {
        File {
            #[serde(rename = "file")]
            path: String,
        },
        Directory {
            #[serde(rename = "directory")]
            path: String,
            media: crate::types::files::Media,
            recursive: bool,
        },
        Id(ItemId),
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Playlist {
        #[serde(rename = "playlistid")]
        pub id: usize,
        #[serde(rename = "type")]
        pub kind: Type,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Type {
        Unknown,
        Video,
        Audio,
        Picture,
        Mixed,
    }
}

pub mod video {
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct CastMember {
        pub name: String,
        pub order: String,
        pub role: String,
        pub thumbnail: Option<String>,
    }

    pub type Cast = Vec<CastMember>;

    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Resume {
        pub position: f64,
        pub total: f64,
    }
}

#[test]
fn test_global_time() {
    use crate::types::global::Time;
    use std::time::Duration;

    let dur = Duration::new(132, 0);
    let time: Time = dur.into();

    assert_eq!(
        time,
        Time {
            hours: 0,
            minutes: 2,
            seconds: 12,
            milliseconds: 0,
        }
    );

    assert_eq!(Duration::from(time), dur);
}
