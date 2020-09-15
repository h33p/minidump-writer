# Microsoft minidump generator written in Rust

Minidump is a commonly used format in analyzing program memory, because it contains a lot of key information about the process, and is tightly integrated in Windows crash handling.

This project aims to be a simple yet powerful minidump file generator, for use in projects that have access to program's memory, and are in need to dump it in a portable format.

At the current stage only a very limited set of streams are available, but that is to be expanded upon.
