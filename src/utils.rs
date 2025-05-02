pub fn check_lyrics_exist(name: &str, artist: &str) -> bool {
    let path = dirs::data_local_dir()
        .unwrap()
        .join("Riri")
        .join("Data")
        .join(format!("{}-{}.xml", name, artist));
    path.exists()
}
