use rkyv::{AlignedVec, Deserialize};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};
use zip::read::ZipArchive;

use crate::logic::system::input::InputFrameEvents;
use crate::logic::system::save::{SaveStateInits, SaveStateUpdates, INPUT_DATA, INPUT_INDEX, STATE_DATA, STATE_INDEX};
use crate::logic::system::state::StateSet;
use crate::utils::{xfrom, xfromf, xres, XResult};

#[derive(Debug)]
pub struct LogicPlayback {
    input_index: Vec<u32>,
    input_file: Option<File>,
    state_index: Vec<[u32; 2]>,
    state_file: Option<File>,

    input_ptr: u32,
    state_ptr: u32,
}

impl LogicPlayback {
    pub fn new<P: AsRef<Path>>(path: P, input: bool, state: bool) -> XResult<LogicPlayback> {
        let mut system = LogicPlayback {
            input_ptr: 0,
            state_ptr: 0,
            input_index: vec![],
            input_file: None,
            state_index: vec![],
            state_file: None,
        };
        if path.as_ref().extension() == Some("zip".as_ref()) {
            system.unzip(path, input, state)?;
        } else {
            system.open(path, input, state)?;
        }
        Ok(system)
    }

    fn unzip<P: AsRef<Path>>(&mut self, zip_path: P, input: bool, state: bool) -> XResult<()> {
        let zip_file = File::open(zip_path.as_ref()).map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;
        let mut zip_archive = ZipArchive::new(zip_file).map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;

        let mut temp_dir = env::temp_dir();
        let folder_name = format!(
            "{}-{}",
            zip_path.as_ref().file_name().unwrap_or_default().to_string_lossy(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        );
        temp_dir.push(folder_name);
        fs::create_dir(&temp_dir).map_err(xfromf!("temp_dir={:?}", temp_dir))?;

        if input {
            let input_index_zip = zip_archive
                .by_name(INPUT_INDEX)
                .map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;
            self.input_index =
                serde_json::from_reader(input_index_zip).map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;

            let mut input_data_zip = zip_archive
                .by_name(INPUT_DATA)
                .map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;
            let input_path = temp_dir.join(INPUT_DATA);
            let mut input_file = File::create(&input_path).map_err(xfromf!("input_path={:?}", input_path))?;
            io::copy(&mut input_data_zip, &mut input_file).map_err(xfromf!("input_path={:?}", input_path))?;
            self.input_file = Some(File::open(&input_path).map_err(xfromf!("input_path={:?}", input_path))?);
        }

        if state {
            let state_index_zip = zip_archive
                .by_name(STATE_INDEX)
                .map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;
            self.state_index =
                serde_json::from_reader(state_index_zip).map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;

            let mut state_data_zip = zip_archive
                .by_name(STATE_DATA)
                .map_err(xfromf!("zip_path={:?}", zip_path.as_ref()))?;
            let state_path = temp_dir.join(STATE_DATA);
            let mut state_file = File::create(&state_path).map_err(xfromf!("state_path={:?}", state_path))?;
            io::copy(&mut state_data_zip, &mut state_file).map_err(xfromf!("state_path={:?}", state_path))?;
            self.input_file = Some(File::open(&state_path).map_err(xfromf!("state_path={:?}", state_path))?);
        }
        Ok(())
    }

    fn open<P: AsRef<Path>>(&mut self, folder_path: P, input: bool, state: bool) -> XResult<()> {
        if input {
            let input_path = folder_path.as_ref().join(INPUT_DATA);
            self.input_file = Some(File::open(&input_path).map_err(xfromf!("input_path={:?}", input_path))?);
        }
        if state {
            let state_path = folder_path.as_ref().join(STATE_DATA);
            self.state_file = Some(File::open(&state_path).map_err(xfromf!("state_path={:?}", state_path))?);
        }
        Ok(())
    }

    pub fn read_input(&mut self) -> XResult<Option<InputFrameEvents>> {
        if let Some(file) = &mut self.input_file {
            if self.input_ptr + 1 >= self.input_index.len() as u32 {
                return Ok(None);
            }

            let data_pos = self.input_index[self.input_ptr as usize];
            let data_end = self.input_index[self.input_ptr as usize + 1];

            let buf_len = (data_end - data_pos) as usize;
            let mut data_buf = AlignedVec::with_capacity(buf_len);
            unsafe {
                data_buf.set_len(buf_len);
            };
            file.read_exact(&mut data_buf).map_err(xfrom!("read exact"))?;

            let archived = unsafe { rkyv::archived_root::<InputFrameEvents>(&data_buf) };
            let mut deserializer = rkyv::Infallible;
            let players_inputs: InputFrameEvents = archived.deserialize(&mut deserializer).unwrap();
            assert_eq!(players_inputs.frame, self.input_ptr + 1); // Input frame starts from 1

            self.input_ptr += 1;
            Ok(Some(players_inputs))
        } else {
            xres!(BadOperation; "file not open")
        }
    }

    pub fn read_state(&mut self) -> XResult<Option<Arc<StateSet>>> {
        if let Some(file) = &mut self.input_file {
            if self.state_ptr + 1 >= self.state_index.len() as u32 {
                return Ok(None);
            }

            let [init_pos, update_pos] = self.state_index[self.state_ptr as usize];
            let [end, _] = self.state_index[self.state_ptr as usize + 1];

            let mut state_inits = SaveStateInits::default();
            let init_buf_len = (update_pos - init_pos) as usize;
            if init_buf_len != 0 {
                let mut init_buf = AlignedVec::with_capacity(init_buf_len);
                unsafe {
                    init_buf.set_len(init_buf_len);
                };
                file.read_exact(&mut init_buf).map_err(xfrom!("read exact"))?;

                let init_archived = unsafe { rkyv::archived_root::<SaveStateInits>(&init_buf) };
                let mut init_deserializer = rkyv::de::deserializers::SharedDeserializeMap::new();
                state_inits = init_archived.deserialize(&mut init_deserializer).unwrap();
                assert_eq!(state_inits.frame, self.state_ptr);
            }

            let update_buf_len = (end - update_pos) as usize;
            let mut update_buf = AlignedVec::with_capacity(update_buf_len);
            unsafe {
                update_buf.set_len(update_buf_len);
            };
            file.read_exact(&mut update_buf).map_err(xfrom!("read exact"))?;

            let update_archived = unsafe { rkyv::archived_root::<SaveStateUpdates>(&update_buf) };
            let mut update_deserializer = rkyv::Infallible;
            let state_updates: SaveStateUpdates = update_archived.deserialize(&mut update_deserializer).unwrap();
            assert_eq!(state_updates.frame, self.state_ptr);

            self.state_ptr += 1;
            Ok(Some(Arc::new(StateSet {
                frame: state_updates.frame,
                inits: state_inits.inits,
                updates: state_updates.updates,
            })))
        } else {
            xres!(BadOperation; "file not open")
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::logic::base::{LogicType, StateAnyBase, StateType};
    use crate::logic::character::{StatePlayerInit, StatePlayerUpdate};
    use crate::logic::game::{StateGameInit, StateGameUpdate};
    use crate::logic::stage::{StateStageInit, StateStageUpdate};
    use crate::logic::system::input::InputPlayerEvents;
    use crate::logic::system::save::SystemSave;
    use crate::logic::StateCharaPhysics;
    use crate::utils::{asb, RawEvent, RawKey};

    #[test]
    fn test_save_input() {
        let _ = fs::remove_dir_all("./test-save-input");
        let _ = fs::remove_file("./test-save-input.zip");

        let mut ss = SystemSave::new("./test-save-input").unwrap();

        let fpes = InputFrameEvents::new(100, &[InputPlayerEvents::new(99, 0, vec![])]);
        assert!(ss.save_input(fpes).is_err());

        let fpes0 = InputFrameEvents::new(
            1,
            &[
                InputPlayerEvents::new(
                    100,
                    0,
                    vec![
                        RawEvent::new_button(RawKey::Attack1, true),
                        RawEvent::new_button(RawKey::Attack1, false),
                    ],
                ),
                InputPlayerEvents::new(101, 0, vec![RawEvent::new_move(Vec2::new(1.0, 1.0).normalize())]),
            ],
        );
        ss.save_input(fpes0.clone()).unwrap();

        let fpes1 = InputFrameEvents::new(
            2,
            &[
                InputPlayerEvents::new(101, 1, vec![RawEvent::new_move(Vec2::ZERO)]),
                InputPlayerEvents::new(
                    100,
                    1,
                    vec![
                        RawEvent::new_button(RawKey::Skill4, true),
                        RawEvent::new_button(RawKey::Skill4, false),
                        RawEvent::new_button(RawKey::Item1, true),
                    ],
                ),
            ],
        );
        ss.save_input(fpes1.clone()).unwrap();

        ss.exit_and_pack();
        thread::sleep(Duration::from_millis(100));

        let mut pb = LogicPlayback::new("./test-save-input.zip", true, false).unwrap();

        let pb_fpes0 = pb.read_input().unwrap();
        assert_eq!(pb_fpes0, Some(fpes0));

        let pb_fpes1 = pb.read_input().unwrap();
        assert_eq!(pb_fpes1, Some(fpes1));

        let pb_fpes2 = pb.read_input().unwrap();
        assert_eq!(pb_fpes2, None);
    }

    #[test]
    fn test_save_state() {
        let _ = fs::remove_dir_all("./test-save-state");
        let _ = fs::remove_file("./test-save-state.zip");

        let mut ss = SystemSave::new("./test-save-state").unwrap();

        let state = Arc::new(StateSet::new(100, 0, 0));
        assert!(ss.save_state(state).is_err());

        let mut state0 = StateSet::new(0, 0, 0);
        state0.inits.push(Arc::new(StateGameInit {
            _base: StateAnyBase::new(1, StateType::GameInit, LogicType::Game),
        }));
        state0.inits.push(Arc::new(StateStageInit {
            _base: StateAnyBase::new(2, StateType::StageInit, LogicType::Stage),
            view_stage_file: "Stage.Demo".into(),
        }));
        let state0 = Arc::new(state0);
        ss.save_state(state0.clone()).unwrap();

        let mut state1 = StateSet::new(1, 0, 0);
        state1.updates.push(Box::new(StateGameUpdate {
            _base: StateAnyBase::new(1, StateType::GameUpdate, LogicType::Game),
            frame: 0,
            id_gen_counter: 123,
        }));
        state1.updates.push(Box::new(StateStageUpdate {
            _base: StateAnyBase::new(2, StateType::StageUpdate, LogicType::Stage),
        }));
        let state1 = Arc::new(state1);
        ss.save_state(state1.clone()).unwrap();

        let mut state2 = StateSet::new(2, 0, 0);
        state2.inits.push(Arc::new(StatePlayerInit {
            _base: StateAnyBase::new(100, StateType::PlayerInit, LogicType::Player),
            skeleton_file: asb!("skel.ozz"),
            animation_files: vec![asb!("anim_stand_idle.ozz"), asb!("anim_stand_ready.ozz")],
            view_model: "model.ozz".into(),
        }));
        state2.updates.push(Box::new(StatePlayerUpdate {
            _base: StateAnyBase::new(100, StateType::PlayerUpdate, LogicType::Player),
            physics: StateCharaPhysics::default(),
            actions: vec![],
        }));
        let state2 = Arc::new(state2);
        ss.save_state(state2.clone()).unwrap();

        ss.exit_and_pack();
        thread::sleep(Duration::from_millis(100));

        let mut pb = LogicPlayback::new("./test-save-state.zip", false, true).unwrap();

        let pb_state0 = pb.read_state().unwrap();
        assert_eq!(pb_state0, Some(state0));

        let pb_state1 = pb.read_state().unwrap();
        assert_eq!(pb_state1, Some(state1));

        let pb_state2 = pb.read_state().unwrap();
        assert_eq!(pb_state2, Some(state2));

        let pb_state3 = pb.read_state().unwrap();
        assert_eq!(pb_state3, None);
    }
}
