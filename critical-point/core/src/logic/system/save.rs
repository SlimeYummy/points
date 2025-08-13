use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::{mem, ptr, u32};
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::logic::base::StateAny;
use crate::logic::system::input::InputFrameEvents;
use crate::logic::system::state::StateSet;
use crate::utils::{xerr, xerrf, xfromf, xres, xresf, XResult};

pub(crate) const SAVE_META: &str = "meta.json";
pub(crate) const INPUT_INDEX: &str = "input_index.json";
pub(crate) const INPUT_DATA: &str = "input_data.rkyvx";
pub(crate) const STATE_INDEX: &str = "state_index.json";
pub(crate) const STATE_DATA: &str = "state_data.rkyvx";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SaveMeta {
    pub game: String,
    pub rule_version: String,
}

#[derive(Debug, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub(crate) struct SaveStateInits {
    pub(crate) frame: u32,
    pub(crate) inits: Vec<Arc<dyn StateAny>>,
}

impl SaveStateInits {
    fn to_rkyv_bytes(frame: u32, inits: &Vec<Arc<dyn StateAny>>) -> Result<rkyv::util::AlignedVec> {
        use rkyv::rancor::Failure;

        let mut save = SaveStateInits { frame, inits: vec![] };
        unsafe {
            ptr::copy_nonoverlapping(inits, &mut save.inits, 1);
        };
        let buf = rkyv::to_bytes::<Failure>(&save)?;
        mem::forget(save);
        Ok(buf)
    }
}

#[derive(Debug, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SaveStateUpdates {
    pub frame: u32,
    pub updates: Vec<Box<dyn StateAny>>,
}

impl SaveStateUpdates {
    fn to_rkyv_bytes(frame: u32, updates: &Vec<Box<dyn StateAny>>) -> Result<rkyv::util::AlignedVec> {
        use rkyv::rancor::Failure;

        let mut save = SaveStateUpdates { frame, updates: vec![] };
        unsafe {
            ptr::copy_nonoverlapping(updates, &mut save.updates, 1);
        };
        let buf = rkyv::to_bytes::<Failure>(&save)?;
        mem::forget(save);
        Ok(buf)
    }
}

enum SaveMessage {
    Input(InputFrameEvents),
    State(Arc<StateSet>),
    Exit(bool),
}

#[derive(Debug)]
pub struct SystemSave {
    thread: Option<JoinHandle<()>>,
    sender: Sender<SaveMessage>,
    input_frame: u32,
    state_frame: u32,
}

impl Drop for SystemSave {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _ = self.sender.send(SaveMessage::Exit(false));
            self.thread = None;
        }
        #[cfg(feature = "debug-print")]
        log::debug!("SystemSave::drop()");
    }
}

impl SystemSave {
    pub fn new<P: AsRef<Path>>(save_path: P) -> XResult<SystemSave> {
        fs::create_dir_all(save_path.as_ref()).map_err(xfromf!("save_path={:?}", save_path.as_ref()))?;

        let input_index_file = Self::create_file(save_path.as_ref(), INPUT_INDEX, Some(b"[0]"))?;
        let input_data_file = Self::create_file(save_path.as_ref(), INPUT_DATA, None)?;

        let state_index_file = Self::create_file(save_path.as_ref(), STATE_INDEX, Some(b"[]"))?;
        let state_data_file = Self::create_file(save_path.as_ref(), STATE_DATA, None)?;

        let mut zip_path = PathBuf::from(save_path.as_ref());
        let name = zip_path
            .file_name()
            .ok_or_else(|| xerrf!(BadArgument; "zip_path={:?}", zip_path))?;
        zip_path.set_file_name(format!("{}.zip", name.to_string_lossy()));
        let save_path = save_path.as_ref().to_path_buf();

        let (sender, receiver) = mpsc::channel();
        let thread = thread::spawn(move || {
            SaveThread {
                receiver,
                input_index_file,
                input_data_file,
                state_index_file,
                state_data_file,
                save_path,
                zip_path,
            }
            .run();
        });

        Ok(SystemSave {
            thread: Some(thread),
            sender,
            input_frame: 0,
            state_frame: u32::MAX,
        })
    }

    fn create_file(dir_path: &Path, name: &str, data: Option<&[u8]>) -> XResult<File> {
        let file_path = dir_path.join(name);
        let res: Result<_, io::Error> = (|| {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&file_path)?;
            if let Some(data) = data {
                file.write(data)?;
                file.seek(SeekFrom::Current(-1))?; // before ']'
            }
            Ok(file)
        })();
        res.map_err(xfromf!("file_path={:?}", file_path))
    }

    pub fn save_input(&mut self, players: InputFrameEvents) -> XResult<()> {
        if self.thread.is_none() {
            return xres!(BadOperation; "thread stopped");
        }
        if players.frame != self.input_frame.wrapping_add(1) {
            return xresf!(BadArgument; "player.frame={}, input_frame={}", players.frame, self.input_frame);
        }
        self.input_frame = players.frame;
        self.sender
            .send(SaveMessage::Input(players))
            .map_err(|_| xerr!(Unexpected; "sender closed"))
    }

    pub fn save_state(&mut self, state_set: Arc<StateSet>) -> XResult<()> {
        if self.thread.is_none() {
            return xres!(BadOperation; "thread stopped");
        }
        if state_set.frame != self.state_frame.wrapping_add(1) {
            return xresf!(BadArgument; "state_set.frame={}, input_frame={}", state_set.frame, self.input_frame);
        }
        self.state_frame = state_set.frame;
        self.sender
            .send(SaveMessage::State(state_set))
            .map_err(|_| xerr!(Unexpected; "sender closed"))
    }

    pub fn save_states(&mut self, mut state_sets: Vec<Arc<StateSet>>) -> XResult<()> {
        for state_set in state_sets.drain(..) {
            self.save_state(state_set)?;
        }
        Ok(())
    }

    pub fn exit_and_pack(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.sender.send(SaveMessage::Exit(true));
            let _ = thread.join();
        }
    }
}

struct SaveThread {
    receiver: Receiver<SaveMessage>,
    input_index_file: File,
    input_data_file: File,
    state_index_file: File,
    state_data_file: File,
    save_path: PathBuf,
    zip_path: PathBuf,
}

impl SaveThread {
    fn run(&mut self) {
        loop {
            let mut exit = false;
            let res = match self.receiver.recv() {
                Ok(SaveMessage::Input(input)) => self.handle_input(input),
                Ok(SaveMessage::State(state_set)) => self.handle_state(state_set),
                Ok(SaveMessage::Exit(pack)) => {
                    exit = true;
                    self.handle_pack(pack)
                }
                Err(err) => Err(err.into()),
            };
            if let Err(err) = res {
                eprintln!("SaveThread::run() {}", err);
            }
            if exit {
                return;
            }
        }
    }

    fn handle_input(&mut self, players: InputFrameEvents) -> Result<()> {
        use rkyv::rancor::Failure;

        let data_buf = rkyv::to_bytes::<Failure>(&players)?;
        self.input_data_file.write_all(&data_buf)?;
        self.input_data_file.flush()?;

        let data_pos = self.input_data_file.stream_position()?;
        if data_pos > u32::MAX as u64 {
            return Err(anyhow!("SystemSave::handle_input() players data file too long"));
        }

        let json = format!(",{}]", data_pos as u32);
        self.input_index_file.write(json.as_bytes())?;
        self.input_index_file.flush()?;
        self.input_index_file.seek(SeekFrom::Current(-1))?;

        // log::debug!("Save input frame({}) size({})", players.frame, data_buf.len());
        Ok(())
    }

    fn handle_state(&mut self, state_set: Arc<StateSet>) -> Result<()> {
        let init_pos = self.state_data_file.stream_position()?;
        if init_pos > u32::MAX as u64 {
            return Err(anyhow!("SystemSave::handle_state() state data file too long"));
        }

        if state_set.inits.len() > 0 {
            let init_buf = SaveStateInits::to_rkyv_bytes(state_set.frame, &state_set.inits)?;
            self.state_data_file.write_all(&init_buf)?;
        }

        let update_pos = self.state_data_file.stream_position()?;
        if update_pos > u32::MAX as u64 {
            return Err(anyhow!("SystemSave::handle_state() state data file too long"));
        }

        let update_buf = SaveStateUpdates::to_rkyv_bytes(state_set.frame, &state_set.updates)?;
        self.state_data_file.write_all(&update_buf)?;
        self.state_data_file.flush()?;

        let tail_pos = self.state_data_file.stream_position()?;
        if tail_pos > u32::MAX as u64 {
            return Err(anyhow!("SystemSave::handle_state() state data file too long"));
        }

        let current_json = format!("[{},{}],", init_pos as u32, update_pos as u32);
        self.state_index_file.write(current_json.as_bytes())?;
        let index_pos = self.state_index_file.stream_position()?;

        let tail_json = format!("[{0},{0}]]", tail_pos as u32);
        self.state_index_file.write(tail_json.as_bytes())?;
        self.state_index_file.flush()?;
        self.state_index_file.seek(SeekFrom::Start(index_pos))?;

        // log::debug!("Save state frame({}) size({})", state_set.frame, update_buf.len());
        Ok(())
    }

    fn handle_pack(&self, pack: bool) -> Result<()> {
        if !pack {
            return Ok(());
        }

        let zip_file = File::create(&self.zip_path)?;
        let mut zip_writer = ZipWriter::new(zip_file);

        let opt = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Zstd)
            .compression_level(Some(11));

        zip_writer.start_file_from_path(SAVE_META, opt)?;
        let meta_json = serde_json::to_vec(&SaveMeta {
            game: String::from("Critical Point"),
            rule_version: String::from("0.1.0"),
        })?;
        zip_writer.write(&meta_json)?;

        for name in [INPUT_INDEX, INPUT_DATA, STATE_INDEX, STATE_DATA] {
            let mut file = File::open(self.save_path.join(name))?;
            zip_writer.start_file_from_path(name, opt)?;
            io::copy(&mut file, &mut zip_writer)?;
        }
        zip_writer.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Tests are in file ../playback.rs
}
