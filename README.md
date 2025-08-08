# mtag

[![Code Style Badge](https://github.com/ethanuppal/mtag/actions/workflows/lint.yaml/badge.svg)](https://github.com/ethanuppal/mtag/blob/main/.github/workflows/lint.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/mtag-tui)](https://crates.io/crates/mtag-tui)

`mtag` is a TUI to turn audio files into an album.
Run it on a folder of audio files and it will create a subfolder called `tagged/` which you can, for example, directly drag into Apple Music.
It'll also cache the metadata in `album.toml` which allows you to easily fix mistakes by rerunning `mtag`.
