use std::fs::OpenOptions;
use std::io::Result;
use std::fs::File as FsFile;
use std::fs::Metadata;
use std::fs::DirBuilder;
use std::io::Write;
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;

pub struct File {
    file: FsFile
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<File> {
        let path_ref = &path.as_ref();
        let parent = &path.as_ref()
            .parent()
            .ok_or(Error::new(ErrorKind::InvalidInput,
                              format!("file {:?} is invaild input ", &path_ref)))?;

        DirBuilder::new().recursive(true).create(parent)?;

        let _ = &path_ref
            .file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput,
                              format!("file {:?} is invalid input,not have file name", &path_ref)));


        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&path)?;

        Ok(File {
            file: file
        })
    }

    pub fn metadata(&self) -> Result<Metadata> {
        self.file.metadata()
    }
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}


impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}


