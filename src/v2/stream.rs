#[deriving(Show)]
pub enum MediaType {
    Audio,
    Video,
    Other,
}

pub struct Stream {
    pub media_type: MediaType,
    pub index: int,
}
