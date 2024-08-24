use ev::MouseEvent;
// use leptos::leptos_dom::ev::SubmitEvent;
#[allow(clippy::wildcard_imports)]
use leptos::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlInputElement};
use tauri_sys::{tauri, dialog};

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
enum PodcastStatus {
    Success,
    Failure,
}

#[derive(Serialize, Clone, Copy)]
struct PodcastArgs<'a> {
    options: &'a PodcastOptions<'a>,
}

fn log(msg: &JsValue) {
    console::log_1(msg); // Logs the message to the browser console
}

fn log_str(msg: &str) {
    let val = JsValue::from_str(msg);
    console::log_1(&val); // Logs the message to the browser console
}

async fn pick_file() -> String {
    let result = dialog::FileDialogBuilder::new();
    let f = result.pick_file().await.unwrap();
    f.unwrap().to_string_lossy().into_owned()
}

async fn pick_directory() -> String {
    let mut result = dialog::FileDialogBuilder::new();
    let f = result.pick_folder().await.unwrap();
    f.unwrap().to_string_lossy().into_owned()
}


async fn open_directory(directory: &str) -> String {
    #[derive(Serialize)]
    struct OpenDirectoryArgs<'a> {
        directory: &'a str,
    }
    tauri::invoke("open_directory", &OpenDirectoryArgs { directory }).await.unwrap()
}

async fn get_file_path (ev: MouseEvent) -> String {
    let target = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
    let file_path = pick_file().await;
    log_str(&format!("file path: {file_path}"));
    target.set_value(&file_path);
    file_path
}

async fn get_directory_path (ev: MouseEvent) -> String {
    let target = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
    let directory_path = pick_directory().await;
    log_str(&format!("directory path: {directory_path}"));
    target.set_value(&directory_path);
    directory_path
}


#[component]
#[allow(clippy::module_name_repetitions, clippy::too_many_lines)]
pub fn App() -> impl IntoView {
    let (episode, set_episode) = create_signal(String::from("My Episode"));
    let (title, set_title) = create_signal(String::from("My Title"));
    let (author, set_author) = create_signal(String::from("My Name"));
    let (podcast_name, set_podcast_name) = create_signal(String::from("My Podcast"));
    let (genre, set_genre) = create_signal(String::from("Spoken Work"));
    let (description, set_description) = create_signal(String::new());
    let (guuid, set_guuid) = create_signal(String::from("https://example.com"));
    let (episode_number, set_episode_number) = create_signal(String::from("1"));

    let (input_file, set_input_file) = create_signal(String::from(
        "/Users/admin/workspace/podcast-master/example/a.wav",
    ));
    let (artwork_file, set_artwork_file) = create_signal(String::from(
        "/Users/admin/workspace/podcast-master/example/a-artwork.webp",
    ));
    let (output_directory, set_output_directory) =
        create_signal(String::from("/Users/admin/workspace/podcast-master/output"));

    let prepare_podcast = move |_| {
        log_str("Preparing podcast");
        spawn_local(async move {
            log_str("Preparing inside move closure podcast");
            let podcast_options = PodcastOptions {
                input_file: &input_file.get_untracked(),
                artwork_file: &artwork_file.get_untracked(),
                podcast_name: &podcast_name.get_untracked().clone(),
                episode: &episode.get_untracked().clone(),
                title: &title.get_untracked().clone(),
                author: &author.get_untracked().clone(),
                genre: &genre.get_untracked().clone(),
                description: &description.get_untracked().clone(),
                guuid: &guuid.get_untracked().clone(),
                episode_number: &episode_number.get_untracked().clone(),
                output_directory: &output_directory.get_untracked().clone(),
            };

            let podcast_args = PodcastArgs {
                options: &podcast_options,
            };

            let args = to_value(&podcast_args).unwrap();

            log(&args);

            // Call the prepare_podcast command
            let status: PodcastStatus = tauri::invoke("prepare_podcast", &podcast_args).await.unwrap();
            if status == PodcastStatus::Success {
                log_str("Podcast prepared successfully");
                open_directory(&output_directory.get_untracked()).await;
            } else {
                log_str("Podcast preparation failed");
            }
            log_str("Podcast prepared");
        });
    };

    let update_podcast_name = move |ev| {
        let v = event_target_value(&ev);
        set_podcast_name.set(v);
    };

    let update_title = move |ev| {
        let v = event_target_value(&ev);
        set_title.set(v);
    };

    let update_episode = move |ev| {
        let v = event_target_value(&ev);
        set_episode.set(v);
    };

    let update_author = move |ev| {
        let v = event_target_value(&ev);
        set_author.set(v);
    };

    let update_genre = move |ev| {
        let v = event_target_value(&ev);
        set_genre.set(v);
    };

    let update_description = move |ev| {
        let v = event_target_value(&ev);
        set_description.set(v);
    };

    let update_guuid = move |ev| {
        let v = event_target_value(&ev);
        set_guuid.set(v);
    };

    let update_episode_number = move |ev| {
        let v = event_target_value(&ev);
        set_episode_number.set(v);
    };

    let update_input_file = move |ev: MouseEvent| {
        spawn_local(async move {
            set_input_file.set(get_file_path(ev).await);
        });
    };

    let update_artwork_file = move |ev: MouseEvent| {
        spawn_local(async move {
            set_artwork_file.set(get_file_path(ev).await);
        });
    };

    let update_output_directory = move |ev: MouseEvent| {
        spawn_local(async move {
            set_output_directory.set(get_directory_path(ev).await);
        });
    };

    view! {
        <main class="container">
            <h1>Podcast Master</h1>
            <label for="input_file">Input File</label>
            <input
                type="text"
                id="input_file"
                placeholder="Input File"
                bind:value=input_file
                on:click=update_input_file
            />
            <label for="artwork_file">Artwork File</label>
            <input
                type="text"
                id="artwork_file"
                placeholder="Artwork File"
                bind:value=artwork_file
                on:click=update_artwork_file
            />
            <label for="output_directory">Output Directory</label>
            <input
                type="text"
                id="output_directory"
                placeholder="Output Directory"
                bind:value=output_directory
                on:click=update_output_directory
            />
            <label for="podcast_name">Podcast Name</label>
            <input
                type="text"
                id="podcast_name"
                placeholder="Podcast Name"
                bind:value=podcast_name
                on:input=update_podcast_name
            />
            <label for="title">Title</label>
            <input
                type="text"
                id="title"
                placeholder="Title"
                bind:value=title
                on:input=update_title
            />
            <label for="episode">Episode</label>
            <input
                type="text"
                id="episode"
                placeholder="Episode"
                bind:value=episode
                on:input=update_episode
            />
            <label for="author">Author</label>
            <input
                type="text"
                id="author"
                placeholder="Author"
                bind:value=author
                on:input=update_author
            />
            <label for="genre">Genre</label>
            <input
                type="text"
                id="genre"
                placeholder="Spoken Word"
                bind:value=genre
                on:input=update_genre
            />
            <label for="guuid">GUID</label>
            <input
                type="text"
                id="guuid"
                placeholder="https://example.com"
                bind:value=guuid
                on:input=update_guuid
            />
            <label for="episode_number">Episode Number</label>
            <input
                type="text"
                id="episode_number"
                placeholder="1"
                bind:value=episode_number
                on:input=update_episode_number
            />
            <label for="description">Description</label>
            <textarea
                id="description"
                placeholder="Description"
                bind:value=description
                on:input=update_description
            />
            <br />
            < button
              on:click = prepare_podcast
              id="prepare_podcast"
            >
                Create Podcast
            </button>
        </main>
    }
}
