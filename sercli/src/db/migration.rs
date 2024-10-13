pub enum Migration {
    Create { name: String },
    Add(String),
}
