pub trait StringExt {
    fn to_slug(&self) -> String;
}

impl StringExt for String {
    fn to_slug(&self) -> String {
        self.to_lowercase().replace(" ", "-")
    }
}
