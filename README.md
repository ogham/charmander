# charinfo

charinfo is a program that takes a stream of UTF-8-formatted data on standard input, and displays information about the characters.

- Displays characters with their numeric and hex values
- Contains list of character names
- Highlights invalid UTF-8 input in red

**This currently only works with the nightly, rather than Rust 1.0 stable or 1.1 beta. Sorry about that.**


## Screenshot

![Screenshot of charinfo](screenshot.png)


## Options

- **-b**, **--bytes**: Show index in bytes from 0, rather than characters from 1.
- **-n**, **--names**: Display character names
