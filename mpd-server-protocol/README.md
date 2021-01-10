# mpd-server-protocol

This is a work-in-progress implementation of the MPD server protocol in rust using async networking.

The goal of this crate is to provide an "interface" on top of which one can build a full-featured MPD server.

It does offer:
- Parsing of most MPD commands (goal is 100% protocol compatibility).
- Basic data types to manipulate MPD objects like tags, subsystems, etc
- A `CommandHandler` async-trait that user of this crate will want to implement.

It does not offer:
- A full-featured MPD server.

See `kodi-mpd-proxy` for more information on how to use this crate.
