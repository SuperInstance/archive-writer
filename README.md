# Archive Writer

**A Rust library for writing Unix static library (`.a`/`.lib`) archives** in the standard `ar` format — the same format used by `libfoo.a` files linked into C/C++/Rust binaries.

## Why It Matters

Static libraries are the lingua franca of native compilation. Every `.a` file is an archive containing one or more object files, preceded by a header that tools like `ld`, `ar`, and `cargo` understand. This crate provides a programmatic way to construct those archives without shelling out to the `ar` command — useful in build scripts, custom linkers, package managers, and cross-compilation tooling.

The Unix archive format is delightfully simple: a magic string (`!<arch>\n`) followed by padded 60-byte headers, each describing a name, size, and mode. This crate handles the padding, alignment (entries must be even-byte aligned), and header formatting for you.

## How It Works

The archive format dates to 1970s Unix. Each entry begins with a 60-byte ASCII header containing the member name (16 chars), timestamp (12 chars), owner UID (6 chars), GID (6 chars), file mode (8 chars), and size (10 chars), terminated by a backtick-newline pair. The actual file data follows immediately.

`ArchiveWriter` is generic over any `Write` sink — files, buffers, network streams. You add entries with `add_entry()`, then call `finish()` which writes the magic header, formats each entry's metadata into fixed-width fields using `write!` with left-alignment (`{:<16}` etc.), and copies the raw bytes. If an entry's data length is odd, a padding newline is appended to maintain 2-byte alignment.

## Quick Start

```rust
use archive_writer::ArchiveWriter;

let mut buf = Vec::new();
let mut w = ArchiveWriter::new(&mut buf);

// Add object files
w.add_entry("foo.o", vec![0x7F, 0x45, 0x4C, 0x46]);
w.add_entry("bar.o", vec![0x7F, 0x45, 0x4C, 0x46]);

w.finish().unwrap();

// buf now contains a valid .a file
assert!(buf.starts_with(b"!<arch>\n"));
```

## API

- **`ArchiveEntry`** — A named blob with raw byte data (`name: String`, `data: Vec<u8>`)
- **`ArchiveWriter<W: Write>`** — Builder that accumulates entries and serializes them
  - `new(writer)` — Create from any `Write` sink
  - `add_entry(name, data)` — Queue an object file
  - `finish(self)` — Write the archive and consume the writer

## Architecture Notes

Part of the SuperInstance compilation toolchain. This crate provides the archive-writing primitive used by build tools that need to produce `.a` outputs without depending on the system `ar` binary. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
