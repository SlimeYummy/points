use anyhow::{anyhow, Result};
use rkyv::AlignedVec;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use zip::write::{SimpleFileOptions, ZipWriter};
use static_assertions::assert_eq_size;
use std::{mem, ptr};

use crate::logic::system::input::PlayerKeyEvents;
use crate::logic::system::state::StateSet;
use crate::logic::base::StateAny;
use crate::utils::{AsXResultIO, XError, XResult};

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

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SaveStateInits {
    pub frame: u32,
    pub inits: Vec<Arc<dyn StateAny>>,
}

impl SaveStateInits {
    fn to_rkyv_bytes(frame: u32, inits: &Vec<Arc<dyn StateAny>>) -> Result<AlignedVec> {
        let mut save = SaveStateInits{
            frame,
            inits: vec![],
        };
        unsafe { ptr::copy_nonoverlapping(inits, &mut save.inits, 1);};
        const RKYV_ALLOC: usize = 1024 * 32;
        let buf = rkyv::to_bytes::<_, RKYV_ALLOC>(&save)?;
        mem::forget(save);
        Ok(buf)
    }
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SaveStateUpdates {
    pub frame: u32,
    pub updates: Vec<Box<dyn StateAny>>,
}

impl SaveStateUpdates {
    fn to_rkyv_bytes(frame: u32, updates: &Vec<Box<dyn StateAny>>) -> Result<AlignedVec> {
        let mut save = SaveStateUpdates {
            frame,
            updates: vec![],
        };
        unsafe { ptr::copy_nonoverlapping(updates, &mut save.updates, 1);};
        const RKYV_ALLOC: usize = 1024 * 64;
        let buf = rkyv::to_bytes::<_, RKYV_ALLOC>(&save)?;
        mem::forget(save);
        Ok(buf)
    }
}

enum SaveMessage {
    Input(PlayerKeyEvents),
    State(Arc<StateSet>),
    Exit(bool),
}

#[derive(Debug)]
pub struct SaveSystem {
    thread: Option<JoinHandle<()>>,
    sender: Sender<SaveMessage>,
    input_frame: u32,
    state_frame: u32,
}

impl Drop for SaveSystem {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _ = self.sender.send(SaveMessage::Exit(false));
            self.thread = None;
        }
    }
}

impl SaveSystem {
    pub fn new<P: AsRef<Path>>(save_path: P) -> XResult<SaveSystem> {
        fs::create_dir(save_path.as_ref()).xerr_with(save_path.as_ref())?;

        let input_index_file = Self::create_file(save_path.as_ref(), INPUT_INDEX, Some(b"[0]"))?;
        let input_data_file = Self::create_file(save_path.as_ref(), INPUT_DATA, None)?;

        let state_index_file = Self::create_file(save_path.as_ref(), STATE_INDEX, Some(b"[]"))?;
        let state_data_file = Self::create_file(save_path.as_ref(), STATE_DATA, None)?;

        let mut zip_path = PathBuf::from(save_path.as_ref());
        let name = zip_path
            .file_name()
            .ok_or_else(|| XError::bad_argument("SaveSystem::new() save_path"))?;
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

        Ok(SaveSystem {
            thread: Some(thread),
            sender,
            input_frame: 0,
            state_frame: 0,
        })
    }

    fn create_file(dir_path: &Path, name: &str, data: Option<&[u8]>) -> XResult<File> {
        let file_path = dir_path.join(name);
        let res = (|| {
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
        res.xerr_with(&file_path)
    }

    pub fn save_input(&mut self, events: PlayerKeyEvents) -> XResult<()> {
        if self.thread.is_none() {
            return Err(XError::invalid_operation("SaveSystem::save_input() thread stopped"));
        }
        if events.frame != self.input_frame + 1 {
            return Err(XError::bad_argument("SaveSystem::save_input() events.frame"));
        }
        self.input_frame = events.frame;
        self.sender
            .send(SaveMessage::Input(events))
            .map_err(|_| XError::unexpected("SaveSystem::save_input() receiver closed"))
    }

    pub fn save_inputs(&mut self, players_events: &[PlayerKeyEvents]) -> XResult<()> {
        for events in players_events {
            self.save_input(events.clone())?;
        }
        Ok(())
    }

    pub fn save_state(&mut self, state_set: Arc<StateSet>) -> XResult<()> {
        if self.thread.is_none() {
            return Err(XError::invalid_operation("SaveSystem::save_state() thread stopped"));
        }
        if state_set.frame != self.state_frame + 1 {
            return Err(XError::bad_argument("SaveSystem::save_state() state_set.frame"));
        }
        self.state_frame = state_set.frame;
        self.sender
            .send(SaveMessage::State(state_set))
            .map_err(|_| XError::unexpected("SaveSystem::save_state() receiver closed"))
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
            let res = match self.receiver.recv() {
                Ok(SaveMessage::Input(events)) => self.handle_input(events),
                Ok(SaveMessage::State(state_set)) => self.handle_state(state_set),
                Ok(SaveMessage::Exit(pack)) => {
                    if pack {
                        self.handle_pack();
                    }
                    return;
                }
                Err(err) => Err(err.into()),
            };
            if let Err(err) = res {
                eprintln!("SaveThread::run() {}", err);
            }
        }
    }

    fn handle_input(&mut self, events: PlayerKeyEvents) -> Result<()> {
        const RKYV_ALLOC: usize = 1024 * 8;
        let data_buf = rkyv::to_bytes::<_, RKYV_ALLOC>(&events)?;
        self.input_data_file.write_all(&data_buf)?;

        let data_pos = self.input_data_file.stream_position()?;
        if data_pos > u32::MAX as u64 {
            return Err(anyhow!("SaveSystem::handle_input() input data file too long"));
        }

        let json = format!(",{}]", data_pos as u32);
        self.input_index_file.write(json.as_bytes())?;
        self.input_index_file.seek(SeekFrom::Current(-1))?;
        Ok(())
    }

    fn handle_state(&mut self, state_set: Arc<StateSet>) -> Result<()> {
        let init_pos = self.state_data_file.stream_position()?;
        if init_pos > u32::MAX as u64 {
            return Err(anyhow!("SaveSystem::handle_state() state data file too long"));
        }

        if state_set.inits.len() > 0 {
            let init_buf = SaveStateInits::to_rkyv_bytes(state_set.frame, &state_set.inits)?;
            self.state_data_file.write_all(&init_buf)?;
        }

        let update_pos = self.state_data_file.stream_position()?;
        if update_pos > u32::MAX as u64 {
            return Err(anyhow!("SaveSystem::handle_state() state data file too long"));
        }

        let update_buf = SaveStateUpdates::to_rkyv_bytes(state_set.frame, &state_set.updates)?;
        self.state_data_file.write_all(&update_buf)?;

        let tail_pos = self.state_data_file.stream_position()?;
        if tail_pos > u32::MAX as u64 {
            return Err(anyhow!("SaveSystem::handle_state() state data file too long"));
        }

        let current_json = format!(",[{},{}],", init_pos as u32, update_pos as u32);
        self.state_index_file.write(current_json.as_bytes())?;
        let index_pos = self.state_index_file.stream_position()?;

        let tail_json = format!(",[{0},{0}]]", tail_pos as u32);
        self.state_index_file.write(tail_json.as_bytes())?;
        self.state_index_file.seek(SeekFrom::Start(index_pos))?;
        Ok(())
    }

    fn handle_pack(&self) -> Result<()> {
        let zip_file = File::create(&self.zip_path)?;
        let mut zip_writer = ZipWriter::new(zip_file);

        let opt = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Zstd);

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
