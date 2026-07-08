mod auto_gen;

mod your_code;

#[cfg(all(not(target_feature = "atomics"), target_family = "wasm"))]
#[global_allocator]
static TALC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

#[unsafe(no_mangle)]
pub extern "C" fn get_error_message() -> u64 {
    use critical_point_wasm_types::{HostError, PackReturn};
    (HostError::buffer().len(), HostError::buffer().as_mut_ptr()).pack()
}
