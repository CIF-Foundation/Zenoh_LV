use std::ffi::{CStr, c_char, c_int};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::Write;

// Explicit Zenoh 1.7.2 imports
use zenoh::Session;
use zenoh::config::Config;
use zenoh::pubsub::Subscriber;
use zenoh::Wait; 
use zenoh::liveliness::LivelinessToken;
use zenoh::query::Queryable;
use zenoh::query::Query;
use zenoh::handlers::FifoChannelHandler;

// --- Data Structures ---

pub struct ReaderContext {
    pub payload: Vec<u8>,
    pub new_data: bool,
}

pub struct ReaderHandle {
    pub context: Arc<Mutex<ReaderContext>>,
    pub subscriber: Subscriber<()>, 
}

// --- Helper Functions ---

fn log(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("d:\\dev\\zenoh\\wrapper\\zenoh_debug.log") 
    {
        let _ = writeln!(file, "{}", msg);
    }
}

// --- Session Management ---

#[no_mangle]
pub extern "C" fn zenoh_open(
    config_input: *const c_char,
    session_out: *mut *mut Session, // LabVIEW passes a pointer to a U64
) -> i32 {
    // Initialize the output pointer to null
    if !session_out.is_null() {
        unsafe { *session_out = ptr::null_mut(); }
    } else {
        return -1; // Error: session_out pointer is null
    }

    let mut config = Config::default();

    if !config_input.is_null() {
        let input = unsafe { CStr::from_ptr(config_input) }.to_string_lossy();
        let input = input.trim();

        if !input.is_empty() {
            // Check if it's a file path
            if input.ends_with(".json") || input.ends_with(".json5") {
                match Config::from_file(input) {
                    Ok(file_config) => config = file_config,
                    Err(_) => return -2, // Error: Failed to load/find file
                }
            } 
            // Treat as JSON5 string
            else {
                match Config::from_json5(input) {
                    Ok(json_config) => config = json_config,
                    Err(_) => return -3, // Error: Invalid JSON5 syntax
                }
            }
        }
    }

    // Attempt to open the Zenoh session
    match zenoh::open(config).wait() {
        Ok(session) => {
            unsafe { *session_out = Box::into_raw(Box::new(session)); }
            0 // Success
        },
        Err(_) => -4, // Error: Network/Protocol error during open
    }
}


// #[no_mangle]
// pub extern "C" fn zenoh_open() -> *mut Session {
//     let config = Config::default();
//     match zenoh::open(config).wait() {
//         Ok(session) => Box::into_raw(Box::new(session)),
//         Err(_) => ptr::null_mut(),
//     }
// }

#[no_mangle]
pub extern "C" fn zenoh_close(session_ptr: *mut Session) {
    if !session_ptr.is_null() {
        unsafe { drop(Box::from_raw(session_ptr)) };
    }
}

// --- Putting Data ---
#[no_mangle]
pub extern "C" fn zenoh_put_raw(
    session_ptr: *const Session,
    key: *const c_char,
    data: *const u8,
    data_len: usize,
) -> c_int {
    if session_ptr.is_null() { return -1; }
    let session = unsafe { &*session_ptr };
    let key = unsafe { CStr::from_ptr(key) }.to_string_lossy();
    let buffer = unsafe { std::slice::from_raw_parts(data, data_len) };

    match session.put(key.as_ref(), buffer).wait() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// --- Subscriber / Reader logic ---
// --- Start the reader as thread running in background ---
#[no_mangle]
pub extern "C" fn zenoh_read_start(session_ptr: *const Session, key: *const c_char) -> *mut ReaderHandle {
    if session_ptr.is_null() { return ptr::null_mut(); }
    let session = unsafe { &*session_ptr };
    let key = unsafe { CStr::from_ptr(key) }.to_string_lossy();

    let context = Arc::new(Mutex::new(ReaderContext {
        payload: Vec::new(),
        new_data: false,
    }));

    let cb_context = Arc::clone(&context);
    let subscriber = session
        .declare_subscriber(key.as_ref())
        .callback(move |sample| {
            if let Ok(mut ctx) = cb_context.lock() {
                ctx.payload = sample.payload().to_bytes().to_vec();
                ctx.new_data = true;
            }
        })
        .wait();

    match subscriber {
        Ok(sub) => Box::into_raw(Box::new(ReaderHandle {
            context,
            subscriber: sub,
        })),
        Err(_) => ptr::null_mut(),
    }
}

// --- Poll the the reader thread for new data ---
#[no_mangle]
pub extern "C" fn zenoh_read_poll(
    handle_ptr: *mut ReaderHandle,
    buffer: *mut u8,
    buffer_max: usize,
    actual_len: *mut usize,
) -> c_int {
    if handle_ptr.is_null() { return 0; }
    let handle = unsafe { &*handle_ptr };

    if let Ok(mut ctx) = handle.context.lock() {
        if !ctx.new_data { return 0; }

        let len = ctx.payload.len().min(buffer_max);
        unsafe {
            std::ptr::copy_nonoverlapping(ctx.payload.as_ptr(), buffer, len);
            if !actual_len.is_null() { *actual_len = len; }
        }
        ctx.new_data = false;
        1
    } else {
        0
    }
}

// --- Stop the the reader thread ---
#[no_mangle]
pub extern "C" fn zenoh_read_stop(handle_ptr: *mut ReaderHandle) {
    if !handle_ptr.is_null() {
        unsafe { drop(Box::from_raw(handle_ptr)) };
    }
}

// ---  Setup In-Memory Storage ---
// Returns a handle to the Queryable that must be closed later.
#[no_mangle]
pub extern "C" fn zenoh_storage_start(
    session_ptr: *const Session,
    key_expr: *const c_char,
) -> *mut Queryable<FifoChannelHandler<Query>> {
    if session_ptr.is_null() { return ptr::null_mut(); }
    let session = unsafe { &*session_ptr };
    let key = unsafe { CStr::from_ptr(key_expr) }.to_string_lossy();

    match session.declare_queryable(key.as_ref()).wait() {
        Ok(q) => Box::into_raw(Box::new(q)),
        Err(_) => ptr::null_mut(),
    }
}

// // --- Write Data to In-Memory Storage ---
// // This populates the local "memory" of the Zenoh session for the given key.
// #[no_mangle]
// pub extern "C" fn zenoh_storage_put(
//     session_ptr: *const Session,
//     key: *const c_char,
//     data: *const u8,
//     data_len: usize,
// ) -> c_int {
//     if session_ptr.is_null() { return -1; }
//     let session = unsafe { &*session_ptr };
//     let key_str = unsafe { CStr::from_ptr(key) }.to_string_lossy();
//     let buffer = unsafe { std::slice::from_raw_parts(data, data_len) };

//     match session.put(key_str.as_ref(), buffer).wait() {
//         Ok(_) => 0,
//         Err(_) => -1,
//     }
// }

// --- Query Keys ---
#[no_mangle]
pub extern "C" fn zenoh_query_kv_bytes(
    session_ptr: *const Session,
    key_expr: *const c_char,
    keys_out: *mut u8,          // Flat buffer for Key strings
    keys_max_size: usize,
    key_offsets_out: *mut u32,  // Start index for each Key
    vals_out: *mut u8,          // Flat buffer for raw Payloads
    vals_max_size: usize,
    val_offsets_out: *mut u32,  // Start index for each Value
    max_count: usize,           // Max number of pairs (size of offset arrays)
    timeout_ms: u64,
) -> i32 {
    if session_ptr.is_null() { return -1; }
    let session = unsafe { &*session_ptr };
    let filter = unsafe { CStr::from_ptr(key_expr) }.to_string_lossy();

    let mut k_offset: usize = 0;
    let mut v_offset: usize = 0;
    let mut count: usize = 0;

    if let Ok(receiver) = session.get(filter.as_ref()).wait() {
        while let Ok(maybe_reply) = receiver.recv_timeout(Duration::from_millis(timeout_ms)) {
            if let Some(reply) = maybe_reply {
                if let Ok(sample) = reply.result() {
                    let key_bytes = sample.key_expr().as_str().as_bytes();
                    let val_bytes = sample.payload().to_bytes();

                    // Safety Checks: Stop if we hit any buffer limit
                    if count >= max_count - 1 || 
                       (k_offset + key_bytes.len()) > keys_max_size || 
                       (v_offset + val_bytes.len()) > vals_max_size 
                    {
                        break; 
                    }

                    unsafe {
                        // Store Offsets
                        *key_offsets_out.add(count) = k_offset as u32;
                        *val_offsets_out.add(count) = v_offset as u32;

                        // Copy Key Bytes
                        std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), keys_out.add(k_offset), key_bytes.len());
                        // Copy Value Bytes
                        std::ptr::copy_nonoverlapping(val_bytes.as_ptr(), vals_out.add(v_offset), val_bytes.len());
                    }

                    k_offset += key_bytes.len();
                    v_offset += val_bytes.len();
                    count += 1;
                }
            } else { break; }
        }
    }

    // Write terminal offsets to allow Length = Offset[i+1] - Offset[i]
    unsafe {
        *key_offsets_out.add(count) = k_offset as u32;
        *val_offsets_out.add(count) = v_offset as u32;
    }

    count as i32
}

// --- Remove Data from Storage ---
#[no_mangle]
pub extern "C" fn zenoh_storage_delete(
    session_ptr: *const Session,
    key: *const c_char,
) -> c_int {
    if session_ptr.is_null() { return -1; }
    let session = unsafe { &*session_ptr };
    let key_str = unsafe { CStr::from_ptr(key) }.to_string_lossy();

    match session.delete(key_str.as_ref()).wait() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// --- Tear Down Storage ---
#[no_mangle]
pub extern "C" fn zenoh_storage_stop(
    queryable_ptr: *mut Queryable<FifoChannelHandler<Query>>
) {
    if !queryable_ptr.is_null() {
        unsafe { drop(Box::from_raw(queryable_ptr)) };
    }
}


// ---  Register Liveliness for the LabVIEW node ---
#[no_mangle]
pub extern "C" fn zenoh_declare_liveliness(
    session_ptr: *const Session,
    key_expr: *const c_char,
) -> *mut LivelinessToken {
    if session_ptr.is_null() { return ptr::null_mut(); }
    let session = unsafe { &*session_ptr };
    let key = unsafe { CStr::from_ptr(key_expr) }.to_string_lossy();

    // In 1.7.2, .wait() is used for the synchronous result
    match session.liveliness().declare_token(key.as_ref()).wait() {
        Ok(token) => Box::into_raw(Box::new(token)),
        Err(_) => ptr::null_mut(),
    }
}

// ---  Unregister Liveliness for the LabVIEW node and tell the network the node is now OFFLINE ---
#[no_mangle]
pub extern "C" fn zenoh_undeclare_liveliness(token_ptr: *mut LivelinessToken) {
    if !token_ptr.is_null() {
        unsafe { drop(Box::from_raw(token_ptr)) };
    }
}

// ---  Scan Network for Nodes (Liveliness Query) ---
// Returns a semicolon-delimited string of detected node keys.
#[no_mangle]
pub extern "C" fn zenoh_scan_nodes(
    session_ptr: *const Session,
    key_filter: *const c_char,
    buffer_out: *mut c_char,
    max_buffer_size: c_int,
    timeout_ms: u64,
) -> c_int {
    if session_ptr.is_null() { return -1; }
    let session = unsafe { &*session_ptr };
    let filter = unsafe { CStr::from_ptr(key_filter) }.to_string_lossy();

    let mut active_nodes = Vec::new();
    
    // Perform a liveliness query using the 1.7.2 pattern
    if let Ok(receiver) = session.liveliness().get(filter.as_ref()).wait() {
        while let Ok(maybe_reply) = receiver.recv_timeout(Duration::from_millis(timeout_ms)) {
            if let Some(reply) = maybe_reply {
                if let Ok(sample) = reply.result() {
                    active_nodes.push(sample.key_expr().to_string());
                }
            } else {
                break; // End of Stream
            }
        }
    }

    let joined = active_nodes.join(";");
    let joined_bytes = joined.as_bytes();

    if joined_bytes.len() >= max_buffer_size as usize {
        return -2;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(joined_bytes.as_ptr(), buffer_out as *mut u8, joined_bytes.len());
        *buffer_out.add(joined_bytes.len()) = 0; // Null terminator
    }

    joined_bytes.len() as c_int
}

