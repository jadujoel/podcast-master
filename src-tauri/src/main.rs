// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Podcast Options
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug)]
struct PodcastOptions<'a> {
    input_file: &'a str,
    artwork_file: &'a str,
    episode: &'a str,
    title: &'a str,
    author: &'a str,
    podcast_name: &'a str,
    genre: &'a str,
    description: &'a str,
    guuid: &'a str,
    episode_number: &'a str,
    output_directory: &'a str,
}

// implement default clone for PodcastOptions

#[tauri::command]
fn example() {
    // Prepare the podcast
    prepare_podcast(PodcastOptions {
        input_file: "/Users/admin/workspace/podcast-master/example/a.wav",
        artwork_file: "/Users/admin/workspace/podcast-master/example/a-artwork.webp",
        episode: "my-episode",
        title: "my-title",
        author: "your-name",
        podcast_name: "my-podcast",
        genre: "Spoken Word",
        description: "my-description",
        guuid: "my-guuid",
        episode_number: "1",
        output_directory: "/Users/admin/workspace/podcast-master/output",
    });
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
enum PodcastStatus {
    Success,
    Failure,
}

#[tauri::command]
#[allow(clippy::too_many_lines)]
fn prepare_podcast(options: PodcastOptions) -> PodcastStatus {
    eprintln!("Preparing podcast");

    let PodcastOptions {
        input_file,
        artwork_file,
        episode,
        title,
        author,
        podcast_name,
        genre,
        description,
        guuid,
        episode_number,
        output_directory,
    } = options;

    eprintln!("Preparing podcast: {podcast_name} - {episode_number}");
    eprintln!("Options: {options:?}");

    let directory = format!("{output_directory}/{podcast_name}-{episode_number}");
    let apple_podcast_directory = format!("{directory}/apple-podcast");
    let artwork_outfile_png = format!("{directory}/{episode}.png");
    let artwork_outfile_jpg = format!("{directory}/{episode}.jpg");
    let audio_outfile = format!("{apple_podcast_directory}/{episode}.mp4"); // Output file format changed to .m4a (AAC-LC)

    // Ensure the output directory exists
    let result = std::fs::create_dir_all(&apple_podcast_directory);
    if let Err(e) = result {
        eprintln!("Failed to create output directory: {e}");
        return PodcastStatus::Failure;
    }

    // prepare the artwork
    {
        if run_ffmpeg(vec![
            "-y",
            "-hide_banner",
            "-nostats",
            "-loglevel",
            "error",
            "-i",
            &artwork_file,
            "-vf",
            "scale=3000:3000", // highest resolution for Apple Podcasts
            artwork_outfile_png.as_str(),
        ]) == PodcastStatus::Failure
        {
            return PodcastStatus::Failure;
        }
        if run_ffmpeg(vec![
            "-y",
            "-hide_banner",
            "-nostats",
            "-loglevel",
            "error",
            "-i",
            &artwork_file,
            "-vf",
            "scale=1400:1400", // minimum 1400x1400 for Apple Podcasts
            artwork_outfile_jpg.as_str(),
        ]) == PodcastStatus::Failure
        {
            return PodcastStatus::Failure;
        }
    }

    // prepare apple podcast
    {
        // Prepare the ffmpeg arguments with metadata, artwork, encoding settings, loudness normalization, and a limiter
        let title_str = format!("title={title}");
        let author_str = format!("artist={author}");
        let podcast_name_str = format!("album={podcast_name}");
        let episode_number_str = format!("track={episode_number}");
        let genre_str = format!("genre={genre}");
        let description_str = format!("comment={description}");
        let guuid_str = format!("podcast_id={guuid}");
        let args = vec![
            "-y",
            "-hide_banner",
            "-nostats",
            "-loglevel",
            "error",
            // Input audio file
            "-i",
            &input_file,
            // Input artwork (to embed as cover)
            "-i",
            &artwork_outfile_jpg, // Use the resized artwork as input for embedding
            // Map the streams: 0 = audio, 1 = artwork
            "-map",
            "0:a", // Map the audio stream from the input audio file
            "-map",
            "1:v", // Map the artwork stream from the input artwork file
            // Metadata arguments
            "-metadata",
            &title_str,
            "-metadata",
            &author_str,
            "-metadata",
            &podcast_name_str,
            "-metadata",
            &episode_number_str,
            "-metadata",
            &genre_str,
            "-metadata",
            &description_str,
            "-metadata",
            &guuid_str,
            // Add metadata for the artwork stream
            "-metadata:s:v",
            "title=\"Album cover\"",
            "-metadata:s:v",
            "comment=\"Cover (front)\"",
            // Audio encoding settings for AAC-LC, Apple Podcast specs
            "-c:a",
            "aac", // Set codec to AAC-LC
            "-b:a",
            "128k", // Set bitrate to 128 kbps (Apple Podcast standard for stereo)
            "-ar",
            "44100", // Set sample rate to 44.1 kHz
            "-ac",
            "2", // Set to stereo audio (2 channels)
            "-movflags",
            "+faststart", // Enable fast start for better streaming
            // Encode the artwork as a still image (to be embedded)
            // "-c:v", "jpeg",             // Use PNG encoding for the still image artwork

            // Loudness normalization (target -16 LUFS with peak at -1.5 dBTP)
            "-filter:a",
            "dynaudnorm, loudnorm=I=-16:LRA=11:TP=-1.5, alimiter=limit=0.79",
            // Output format and file
            &audio_outfile,
        ];
        if run_ffmpeg(args) == PodcastStatus::Failure {
            return PodcastStatus::Failure;
        }
    }
    PodcastStatus::Success
}

#[tauri::command]
fn open_directory(directory: &str) -> PodcastStatus {
    if let Err(e) = open::that(directory) {
        eprintln!("Failed to open directory: {e}");
        return PodcastStatus::Failure;
    }

    PodcastStatus::Success
}

// in args allow for any number of arguments
// maybe by using a Vec<&str> instead of [&str]
fn run_ffmpeg(args: Vec<&str>) -> PodcastStatus {
    eprintln!("Running ffmpeg with args: {args:?}");

    let Ok(sidecar) = tauri::api::process::Command::new_sidecar("ffmpeg") else {
        eprintln!("Failed to launch ffmpeg sidecar");
        return PodcastStatus::Failure;
    };

    // Run the ffmpeg command
    let output = sidecar.args(args).output();

    let Ok(output) = output else {
        return PodcastStatus::Failure;
    };

    // Process the output
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    let status = output.status;

    // Print the output
    if !stdout.is_empty() {
        eprintln!("ffmpeg stdout: {stdout}");
    }
    if !stderr.is_empty() {
        eprintln!("ffmpeg stderr: {stderr}");
    }

    if let Some(status_code) = status.code() {
        if status_code != 0 {
            eprintln!("ffmpeg failed with status: {status_code}");
            return PodcastStatus::Failure;
        }
    } else {
        eprintln!("ffmpeg terminated unexpectedly.");
        return PodcastStatus::Failure;
    }
    PodcastStatus::Success
}

fn main() {
    // example();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            prepare_podcast,
            example,
            open_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
