use url::Url;

pub trait YoutubeIdExtractor {
    fn get_video_id(&self) -> Option<String>;
}

impl YoutubeIdExtractor for Url {
    fn get_video_id(&self) -> Option<String> {
        self.query_pairs().find_map(|(key, value)| {
            if key == "v" {
                Some(value.to_string())
            } else {
                None
            }
        })
    }
}
