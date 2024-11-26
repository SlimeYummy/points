use anyhow::Result;
use rkyv::{AlignedVec, Deserialize};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};
use zip::read::ZipArchive;

use crate::logic::base::StateAny;
use crate::logic::system::input::PlayerKeyEvents;
use crate::logic::system::save::{INPUT_DATA, INPUT_INDEX, STATE_DATA, STATE_INDEX};
use crate::logic::system::state::StateSet;
use crate::utils::{AsXResultIO, XError, XResult};

enum ReadSignal {
    Input,
    State,
    Exit,
}

enum ReadData<T> {
    Data(T),
    EOF,
}

#[derive(Debug)]
pub struct LogicPlayback {
    input_index: Vec<u32>,
    input_file: Option<File>,
    state_index: Vec<[u32; 2]>,
    state_file: Option<File>,

    input_frame: u32,
    state_frame: u32,
}

impl LogicPlayback {
    pub fn new<P: AsRef<Path>>(zip_path: P, input: bool, state: bool) -> XResult<LogicPlayback> {
        let mut system = LogicPlayback {
            input_frame: 0,
            state_frame: 0,
            input_index: vec![],
            input_file: None,
            state_index: vec![],
            state_file: None,
        };
        system.unzip(zip_path, input, state)?;
        Ok(system)
    }

    fn unzip<P: AsRef<Path>>(&mut self, zip_path: P, input: bool, state: bool) -> XResult<()> {
        let zip_file = File::create(zip_path.as_ref()).xerr_with(zip_path.as_ref())?;
        let mut zip_archive = ZipArchive::new(zip_file)?;

        let mut temp_dir = env::temp_dir();
        let folder_name = format!(
            "{}-{}",
            zip_path.as_ref().file_name().unwrap_or_default().to_string_lossy(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        );
        temp_dir.push(folder_name);
        fs::create_dir(&temp_dir).xerr_with(&temp_dir)?;

        if input {
            let input_index_zip = zip_archive.by_name(INPUT_INDEX)?;
            self.input_index = serde_json::from_reader(input_index_zip)?;

            let mut input_data_zip = zip_archive.by_name(INPUT_DATA)?;
            let input_path = temp_dir.join(INPUT_DATA);
            let mut input_file = File::create(&input_path).xerr_with(&input_path)?;
            io::copy(&mut input_data_zip, &mut input_file).xerr_with(&input_path)?;
            self.input_file = Some(input_file);
        }

        if state {
            let state_index_zip = zip_archive.by_name(STATE_INDEX)?;
            self.state_index = serde_json::from_reader(state_index_zip)?;

            let mut state_data_zip = zip_archive.by_name(STATE_DATA)?;
            let state_path = temp_dir.join(STATE_DATA);
            let mut state_file = File::create(&state_path).xerr_with(&state_path)?;
            io::copy(&mut state_data_zip, &mut state_file).xerr_with(&state_path)?;
            self.state_file = Some(state_file);
        }
        Ok(())
    }

    fn read_input(&mut self) -> XResult<Option<PlayerKeyEvents>> {
        if let Some(file) = &mut self.input_file {
            if self.input_frame + 1 >= self.input_index.len() as u32 {
                return Ok(None);
            }

            let data_pos = self.input_index[self.input_frame as usize];
            let data_end = self.input_index[self.input_frame as usize + 1];

            let buf_len = (data_end - data_pos) as usize;
            let mut data_buf = AlignedVec::with_capacity(buf_len);
            unsafe {
                data_buf.set_len(buf_len);
            };
            file.read_exact(&mut data_buf).xerr_with(INPUT_DATA)?;

            let archived = unsafe { rkyv::archived_root::<PlayerKeyEvents>(&data_buf) };
            let mut deserializer = rkyv::Infallible;
            let events: PlayerKeyEvents = archived.deserialize(&mut deserializer).unwrap();

            self.input_frame += 1;
            Ok(Some(events))
        } else {
            Err(XError::invalid_operation("LogicPlayback::read_input() "))
        }
    }

    fn read_state(&mut self) -> XResult<Option<Arc<StateSet>>> {
        if let Some(file) = &mut self.input_file {
            if self.state_frame + 1 >= self.state_index.len() as u32 {
                return Ok(None);
            }

            let [init_pos, update_pos] = self.state_index[self.state_frame as usize];
            let [end, _] = self.state_index[self.state_frame as usize + 1];

            let init_buf_len = (update_pos - init_pos) as usize;
            let mut init_buf = AlignedVec::with_capacity(init_buf_len);
            unsafe {
                init_buf.set_len(init_buf_len);
            };
            file.read_exact(&mut init_buf).xerr_with(STATE_DATA)?;

            let init_archived = unsafe { rkyv::archived_root::<Vec<Arc<dyn StateAny>>>(&init_buf) };
            let mut init_deserializer = rkyv::Infallible;
            let inits: Vec<Arc<dyn StateAny>> = init_archived.deserialize(&mut init_deserializer).unwrap();

            let update_buf_len = (end - update_pos) as usize;
            let mut update_buf = AlignedVec::with_capacity(update_buf_len);
            unsafe {
                update_buf.set_len(update_buf_len);
            };
            file.read_exact(&mut update_buf).xerr_with(STATE_DATA)?;

            let update_archived = unsafe { rkyv::archived_root::<Vec<Box<dyn StateAny>>>(&update_buf) };
            let mut update_deserializer = rkyv::Infallible;
            let updates: Vec<Box<dyn StateAny>> = update_archived.deserialize(&mut update_deserializer).unwrap();

            self.state_frame += 1;
            Ok(Some(Arc::new(StateSet {
                frame: self.state_frame,
                inits,
                updates,
            })))
        } else {
            Err(XError::invalid_operation("LogicPlayback::read_input() "))
        }
    }
}
