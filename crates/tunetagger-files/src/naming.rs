use sanitize_filename::sanitize;
use tunetagger_core::ResolvedTags;

pub fn render_filename(template: &str, tags: &ResolvedTags, sanitize_output: bool) -> String {
    let mut name = template
        .replace("{artist}", &tags.artist)
        .replace("{title}", &tags.title)
        .replace("{album}", tags.album.as_deref().unwrap_or(""));

    if sanitize_output {
        name = sanitize(name);
    }

    if !name.to_ascii_lowercase().ends_with(".mp3") {
        name.push_str(".mp3");
    }

    name
}
