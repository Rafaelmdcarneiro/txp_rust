use super::*;
use std::fmt::Display;

impl Display for Mipmap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubTex {}x{} {:?}", self.width, self.height, self.format)
    }
}
