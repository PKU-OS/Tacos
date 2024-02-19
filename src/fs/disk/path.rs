/// Path.
///
/// We uses [`alloc::string::String`] methods for path
/// manipulation.
pub struct Path(alloc::string::String);

impl Path {
    pub fn exists(path: Self) -> bool {
        super::DISKFS.get().root_dir.lock().exists(&path)
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Path(value.into())
    }
}

impl core::ops::Deref for Path {
    type Target = alloc::string::String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
