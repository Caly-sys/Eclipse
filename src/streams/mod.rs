pub mod controller;
pub mod credentials;
pub mod downloader;
pub mod matcher;
pub mod playback;
pub mod provider;
pub mod spotify;
pub mod spotify_public;
pub mod youtube;

pub use controller::{
    StreamAction, StreamCommand, StreamController, StreamEvent,
};
pub use credentials::{
    CredentialStore, SERVICE_SPOTIFY, SERVICE_YOUTUBE, USER_ACCESS_TOKEN, USER_API_KEY,
    USER_CLIENT_ID, USER_REFRESH_TOKEN,
};
pub use downloader::{
    cache_stream_track, get_stream_cache_dir, index_downloaded_track, probe_track_audio_specs,
    silent_command, DownloadJob, DownloadManager, DownloadStatus,
};
pub use matcher::*;
pub use playback::*;
pub use provider::{
    DownloadFormat, PlaybackSource, StreamPlaybackState, StreamPlaylist, StreamProvider,
    StreamTrack,
};
pub use spotify::SpotifyProvider;
pub use spotify_public::*;
pub use youtube::YouTubeProvider;
