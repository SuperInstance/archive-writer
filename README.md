# Archive Writer

**A Rust library for writing Unix static library (`.a`/`.lib`) archives** in the standard `ar` format — the same format used by `libfoo.a` files linked into C/C++/Rust binaries. This crate provides a programmatic way to construct those archives without shelling out to the `ar` command.

## Why It Matters

Static libraries are the lingua franca of native compilation. Every `.a` file is an archive containing one or more object files, preceded by a header that tools like `ld`, `ar`, and `cargo` understand. When `rustc` compiles a `rlib`, it internally constructs an archive. When `gcc` links a C program, it reads `.a` files produced by `ar`.

Programmatic archive construction matters in several scenarios:

- **Build systems** — Cargo, Bazel, and Buck construct archives internally rather than shelling out to `ar`, saving process-creation overhead and enabling cross-compilation
- **Custom linkers** — Linker tools need to read and write archives as first-class data structures
- **Package managers** — Tools that redistribute compiled artifacts need to inspect, modify, and repackage archives
- **Cross-compilation** — When the target platform's `ar` isn't available on the host, a pure Rust implementation works everywhere

The Unix archive format is delightfully simple: a magic string (`!<arch>\n`) followed by padded 60-byte headers, each describing a member's name, size, and mode. This crate handles the padding, alignment, and header formatting for you.

## How It Works

### The Unix Archive Format (System V / GNU variant)

The archive format dates to 1970s Unix (System V). The structure is:

```
!<arch>\n                    ← 8-byte magic header
[header1][data1][padding?]   ← member 1
[header2][data2][padding?]   ← member 2
...
```

Each member header is exactly **60 bytes** of ASCII:

| Offset | Length | Field | Format |
|--------|--------|-------|--------|
| 0 | 16 | Member name | Left-justified, space-padded |
| 16 | 12 | Modification timestamp | Decimal ASCII |
| 28 | 6 | Owner UID | Decimal ASCII |
| 34 | 6 | Group GID | Decimal ASCII |
| 40 | 8 | File mode | Octal ASCII |
| 48 | 10 | File size (bytes) | Decimal ASCII |
| 58 | 2 | End marker | `` `\n `` (backtick + newline) |

**Alignment invariant:** Member data must be at an even byte offset. If a member's data length is odd, a single newline (`\n`) is appended as padding. This 2-byte alignment requirement comes from the original PDP-11 architecture where half-word accesses required even addresses.

### Writing Process

`ArchiveWriter<W: Write>` is generic over any `Write` sink. The writing sequence:

1. **Write magic** — `!<arch>\n` (8 bytes)
2. **For each entry:**
   - Format the 60-byte header using `write!` with left-alignment (`{:<16}`, `{:<12}`, etc.)
   - Write the raw data bytes
   - If data length is odd, write one `\n` padding byte

**Complexity:**

| Operation | Time | Space |
|-----------|------|-------|
| Add entry | O(1) amortized | O(data_size) |
| Finish (write) | O(N × avg_entry_size) | O(total_archive_size) |
| Total | O(Σ entry sizes) | O(total_archive_size) |

The writer accumulates entries in memory (`Vec<ArchiveEntry>`) and serializes them all at once in `finish()`. This is optimal for the common case where all entries are known before writing begins.

### Field Formatting Details

The header fields are left-justified and space-padded to fixed widths. This crate uses Rust's `write!` formatting:

```rust
write!(writer, "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
    name, 0u32, 0u16, 0u16, 0o100644, size)
```

- `name`: truncated/padded to 16 chars (GNU extended naming not supported)
- Timestamps, UID, GID: zeroed (the metadata is cosmetic; linkers ignore it)
- Mode: `100644` octal (regular file, rw-r--r--)
- Size: decimal byte count of the following data

## Quick Start

```rust
use archive_writer::ArchiveWriter;

let mut buf = Vec::new();
let mut w = ArchiveWriter::new(&mut buf);

// Add object files (ELF magic: 0x7F 'E' 'L' 'F')
w.add_entry("foo.o", vec![0x7F, 0x45, 0x4C, 0x46]);
w.add_entry("bar.o", vec![0x7F, 0x45, 0x4C, 0x46]);

w.finish().unwrap();

// buf now contains a valid .a file
assert!(buf.starts_with(b"!<arch>\n"));

// Write to a file
use std::fs::File;
let mut file = File::create("libfoo.a").unwrap();
let mut w = ArchiveWriter::new(&mut file);
w.add_entry("module.o", std::fs::read("module.o").unwrap());
w.finish().unwrap();
```

## API

### `ArchiveEntry`

A named blob with raw byte data.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Member name (truncated to 16 chars) |
| `data` | `Vec<u8>` | Raw member data |

### `ArchiveWriter<W: Write>`

Builder that accumulates entries and serializes them.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(W) → Self` | Create from any `Write` sink |
| `add_entry` | `(&mut self, &str, Vec<u8>)` | Queue an object file |
| `finish` | `(self) → io::Result<()>` | Write archive, consume writer |

## Architecture Notes

Part of the SuperInstance compilation toolchain. This crate provides the archive-writing primitive used by build tools that need to produce `.a` outputs without depending on the system `ar` binary.

Within γ + η = C, the archive format instantiates the conservation law in its alignment invariant: every member's data is padded to even length, conserving the 2-byte alignment property across the entire archive. The total archive size equals the magic (8 bytes) plus the sum of all member headers (60 bytes each) plus all member data plus padding bytes. No bytes are wasted; no alignment is violated. The format is a conserved quantity.

See the [architecture overview](https://github.com/casey-digennaro/archive-writer/blob/main/ARCHITECTURE.md).

## References

1. UNIX System Laboratories (1995). "System V Application Binary Interface." Chapter 7: "Archive File Format."
2. GNU Binutils. `ar.1` man page. (GNU extended format for long filenames)
3. FreeBSD. `ar.5` man page. (BSD variant differences)
4. Rust Compiler. `src/librustc_codegen_ssa/back/linker.rs`. (How rustc uses archives internally)

## License

MIT
