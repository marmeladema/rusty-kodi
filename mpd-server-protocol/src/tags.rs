use enumset::EnumSetType;

#[derive(Debug, EnumSetType)]
pub enum TagType {
    Artist,
    ArtistSort,
    Album,
    AlbumSort,
    AlbumArtist,
    AlbumArtistSort,
    Title,
    Track,
    Name,
    Genre,
    Date,
    OriginalDate,
    Composer,
    Performer,
    Conductor,
    Work,
    Grouping,
    Comment,
    Disc,
    Label,
    MusicBrainzArtistId,
    MusicBrainzAlbumId,
    MusicBrainzAlbumArtistId,
    MusicBrainzTrackId,
    MusicBrainzReleaseTrackId,
    MusicBrainzWorkId,
}

impl TagType {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"artist" => Some(Self::Artist),
            b"artistsort" => Some(Self::ArtistSort),
            b"album" => Some(Self::Album),
            b"albumsort" => Some(Self::AlbumSort),
            b"albumartist" => Some(Self::AlbumArtist),
            b"albumartistsort" => Some(Self::AlbumArtistSort),
            b"title" => Some(Self::Title),
            b"track" => Some(Self::Track),
            b"name" => Some(Self::Name),
            b"genre" => Some(Self::Genre),
            b"date" => Some(Self::Date),
            b"originaldate" => Some(Self::OriginalDate),
            b"composer" => Some(Self::Composer),
            b"performer" => Some(Self::Performer),
            b"conductor" => Some(Self::Conductor),
            b"work" => Some(Self::Work),
            b"grouping" => Some(Self::Grouping),
            b"comment" => Some(Self::Comment),
            b"disc" => Some(Self::Disc),
            b"label" => Some(Self::Label),
            b"musicbrainz_artistid" => Some(Self::MusicBrainzArtistId),
            b"musicbrainz_albumid" => Some(Self::MusicBrainzAlbumId),
            b"musicbrainz_albumartistid" => Some(Self::MusicBrainzAlbumArtistId),
            b"musicbrainz_trackid" => Some(Self::MusicBrainzTrackId),
            b"musicbrainz_releasetrackid" => Some(Self::MusicBrainzReleaseTrackId),
            b"musicbrainz_workid" => Some(Self::MusicBrainzWorkId),
            _ => None,
        }
    }
}

impl std::fmt::Display for TagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagType::Artist => write!(f, "Artist"),
            TagType::ArtistSort => write!(f, "ArtistSort"),
            TagType::Album => write!(f, "Album"),
            TagType::AlbumSort => write!(f, "AlbumSort"),
            TagType::AlbumArtist => write!(f, "AlbumArtist"),
            TagType::AlbumArtistSort => write!(f, "AlbumArtistSort"),
            TagType::Title => write!(f, "Title"),
            TagType::Track => write!(f, "Track"),
            TagType::Name => write!(f, "Name"),
            TagType::Genre => write!(f, "Genre"),
            TagType::Date => write!(f, "Date"),
            TagType::OriginalDate => write!(f, "OriginalDate"),
            TagType::Composer => write!(f, "Composer"),
            TagType::Performer => write!(f, "Performer"),
            TagType::Conductor => write!(f, "Conductor"),
            TagType::Work => write!(f, "Work"),
            TagType::Grouping => write!(f, "Grouping"),
            TagType::Comment => write!(f, "Comment"),
            TagType::Disc => write!(f, "Disc"),
            TagType::Label => write!(f, "Label"),
            TagType::MusicBrainzArtistId => write!(f, "MUSICBRAINZ_ARTISTID"),
            TagType::MusicBrainzAlbumId => write!(f, "MUSICBRAINZ_ALBUMID"),
            TagType::MusicBrainzAlbumArtistId => write!(f, "MUSICBRAINZ_ALBUMARTISTID"),
            TagType::MusicBrainzTrackId => write!(f, "MUSICBRAINZ_TRACKID"),
            TagType::MusicBrainzReleaseTrackId => write!(f, "MUSICBRAINZ_RELEASETRACKID"),
            TagType::MusicBrainzWorkId => write!(f, "MUSICBRAINZ_WORKID"),
        }
    }
}

#[derive(Debug)]
pub struct Tag {
    pub kind: TagType,
    pub value: String,
}

impl Tag {
    pub fn artist(value: String) -> Self {
        Self {
            kind: TagType::Artist,
            value,
        }
    }

    pub fn album(value: String) -> Self {
        Self {
            kind: TagType::Album,
            value,
        }
    }

    pub fn genre(value: String) -> Self {
        Self {
            kind: TagType::Genre,
            value,
        }
    }

    pub fn title(value: String) -> Self {
        Self {
            kind: TagType::Title,
            value,
        }
    }

    pub fn track(value: String) -> Self {
        Self {
            kind: TagType::Track,
            value,
        }
    }

    pub fn disc(value: String) -> Self {
        Self {
            kind: TagType::Disc,
            value,
        }
    }

    pub fn date(value: String) -> Self {
        Self {
            kind: TagType::Date,
            value,
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.kind, self.value)
    }
}

impl From<(TagType, String)> for Tag {
    fn from(value: (TagType, String)) -> Self {
        Self {
            kind: value.0,
            value: value.1,
        }
    }
}
