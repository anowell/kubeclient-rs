

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Json(::serde_json::Error);
        Yaml(::serde_yaml::Error);
        Url(::url::ParseError);
        Http(::reqwest::Error);
    }
}
