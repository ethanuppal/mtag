// Copyright (C) 2025 Ethan Uppal. All rights reserved.

use inquire::{Confirm, Select, Text, ui::RenderConfig};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever, whatever};
use std::{fs, iter, path::Path, process::Command};

mod inquire_stylesheet_shim;
mod prompt_for_path;

#[derive(Serialize, Deserialize, Default)]
struct AlbumMetadata {
    album: Option<String>,
    year: Option<String>,
    genre: Option<String>,
    authors: Vec<String>,
    cover_image: Option<String>,
    #[serde(default)]
    tracks: Vec<TrackMetadata>,
}

#[derive(Serialize, Deserialize, Clone)]
struct TrackMetadata {
    path: String,
    name: String,
    number: String,
}

const METADATA_CACHE_NAME: &str = "album.toml";
const FFMPEG: &str = "ffmpeg";

type Result<T> = std::result::Result<T, Whatever>;

fn check_for_ffmpeg() -> Result<()> {
    if Command::new(FFMPEG).arg("-version").output().is_err() {
        whatever!("ffmpeg is not accessible via $PATH");
    }
    Ok(())
}

fn select_genre(preselect: Option<&str>) -> Result<String> {
    let choices = vec![
        "Video Game",
        "Classical",
        "Jazz",
        "Rock",
        "Metal",
        "Electronic",
        "Funk",
        "Soundtrack",
        "Other",
    ];
    assert!(choices.len() > 0); // since I'm stupid

    Ok(Select::new("Select genre:", choices.clone())
        .with_starting_cursor(
            preselect
                .and_then(|preselect| {
                    choices.iter().position(|item| item == &preselect)
                })
                .unwrap_or(0),
        )
        .prompt()
        .whatever_context("Failed to select genre")?
        .to_string())
}

fn edit_metadata(
    input: &Path,
    output: &Path,
    meta: &TrackMetadata,
    album: &AlbumMetadata,
) -> Result<()> {
    let album_metadata =
        format!("album={}", album.album.as_deref().unwrap_or_default());
    let artist_metadata = format!("artist={}", album.authors.join("; "));
    let title_metadata = format!("title={}", meta.name);
    let track_metadata = format!("track={}", meta.number);
    let date_metadata =
        format!("date={}", album.year.as_deref().unwrap_or_default());
    let genre_metadata =
        format!("genre={}", album.genre.as_deref().unwrap_or_default());

    let mut command = vec!["-y", "-i", input.to_str().unwrap()];

    if let Some(ref cover) = album.cover_image {
        command.push("-i");
        command.push(cover);
        command.extend_from_slice(&["-map", "0:a", "-map", "1:v"]);
    }

    command.extend_from_slice(&["-c", "copy", "-id3v2_version", "3"]);

    command.extend_from_slice(&[
        "-metadata",
        &album_metadata,
        "-metadata",
        &artist_metadata,
        "-metadata",
        &title_metadata,
        "-metadata",
        &track_metadata,
        "-metadata",
        &date_metadata,
        "-metadata",
        &genre_metadata,
    ]);

    if album.cover_image.is_some() {
        command.extend_from_slice(&[
            "-metadata:s:v",
            "title=Album cover",
            "-metadata:s:v",
            "comment=Cover (front)",
        ]);
    }

    command.push(output.to_str().unwrap());

    let _ = fs::remove_file(output);

    let status = Command::new(FFMPEG)
        .args(&command)
        .spawn()
        .whatever_context("Failed to spawn ffmpeg")?
        .wait()
        .whatever_context("Failed to wait for ffmpeg")?;
    if !status.success() {
        whatever!("ffmpeg failed");
    }
    Ok(())
}

#[snafu::report]
fn main() -> Result<()> {
    check_for_ffmpeg()?;

    let render_config: RenderConfig<'static> = inquire::ui::RenderConfig {
        ..Default::default()
    };

    inquire::set_global_render_config(render_config);

    let input_directory = prompt_for_path::prompt_for_path(
        "Choose input directory",
        None,
        &render_config,
    )?;
    let album_metadata_cache_path = input_directory.join(METADATA_CACHE_NAME);
    let mut album_metadata = AlbumMetadata::default();

    if album_metadata_cache_path.exists() {
        let raw_cache_data = fs::read_to_string(&album_metadata_cache_path)
            .whatever_context("Failed to read album metadata cache")?;
        album_metadata = toml::from_str(&raw_cache_data)
            .whatever_context("Failed to parse album metadata cache")?;
    }

    album_metadata.album = Some(
        Text::new("Enter album name:")
            .with_default(album_metadata.album.as_deref().unwrap_or(""))
            .prompt()
            .whatever_context("Failed to get album name")?,
    );

    album_metadata.year = Some(
        Text::new("Enter album year:")
            .with_default(album_metadata.year.as_deref().unwrap_or(""))
            .prompt()
            .whatever_context("Failed to get album year")?,
    );

    let current_genre = album_metadata.genre.clone();
    album_metadata.genre = Some(select_genre(current_genre.as_deref())?);

    let mut authors = Vec::new();
    for existing in album_metadata
        .authors
        .iter()
        .chain(iter::repeat(&String::new()))
    {
        let name = Text::new("Enter author name (blank to finish):")
            .with_default(existing)
            .prompt()
            .whatever_context("Failed to get author name")?;
        if name.trim().is_empty() {
            break;
        }
        authors.push(name);
    }
    album_metadata.authors = authors;

    let add_cover_art = Confirm::new("Add album cover? [y/n]")
        .with_default(true)
        .prompt()
        .whatever_context("Failed to prompt for whether album")?;
    if add_cover_art {
        album_metadata.cover_image = Some(
            prompt_for_path::prompt_for_path(
                "Choose cover image",
                album_metadata.cover_image.as_deref(),
                &render_config,
            )
            .whatever_context("Failed to get cover image path")?
            .display()
            .to_string(),
        );
    }

    let mut tagged_tracks = Vec::new();
    let files = fs::read_dir(&input_directory)
        .whatever_context("Failed to read contents of input directory")?;
    let mut input_mp3s = files
        .filter_map(|f| f.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |ext| {
                ["mp3", "m4a"]
                    .into_iter()
                    .find(|valid| *valid == ext)
                    .is_some()
            })
        })
        .collect::<Vec<_>>();
    input_mp3s.sort();

    if input_mp3s.is_empty() {
        whatever!("{} has no mp3 files", input_directory.display());
    }

    let output_directory = input_directory.join("tagged");
    fs::create_dir_all(&output_directory)
        .whatever_context("Failed to create tagged output directory")?;

    for input_mp3 in input_mp3s {
        let filename = input_mp3.file_name().unwrap().to_string_lossy();
        println!("{filename}:");

        let default_track = album_metadata.tracks.iter().find(|t| {
            Path::new(&t.path)
                .file_name()
                .map(|filename| filename.to_string_lossy())
                == Some(filename.clone())
        });
        let default_name = default_track
            .map(|track| track.name.clone())
            .unwrap_or_default();
        let default_number = default_track
            .map(|track| track.number.clone())
            .unwrap_or_default();

        let name = Text::new("Track title:")
            .with_default(&default_name)
            .prompt()
            .whatever_context("Failed to get track title")?;
        let number = Text::new("Track number:")
            .with_default(&default_number)
            .prompt()
            .whatever_context("Failed to get track number")?;

        let track = TrackMetadata {
            path: input_mp3.to_str().unwrap().to_string(),
            name: name.clone(),
            number: number.clone(),
        };
        let output = output_directory.join(input_mp3.file_name().unwrap());
        edit_metadata(&input_mp3, &output, &track, &album_metadata)?;
        tagged_tracks.push(track);
    }

    album_metadata.tracks = tagged_tracks;

    let toml_encoded_cache = toml::to_string(&album_metadata).unwrap();
    fs::write(album_metadata_cache_path, toml_encoded_cache)
        .whatever_context("Failed to write album metadata cache")?;

    println!(
        "All tracks processed and saved to: {}",
        output_directory.display()
    );

    Ok(())
}
