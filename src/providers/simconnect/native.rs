//! Minimal hand-written ABI from the official MSFS2024 SDK 1.8.1 (build 2603316).
//!
//! Receive structures use the header's `#pragma pack(push, 1)`. Decoding copies
//! bounded byte ranges; it never borrows packed fields or retains DLL buffers.

use std::{
    cell::Cell,
    ffi::{CStr, c_char},
    fmt,
    marker::PhantomData,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr,
    rc::Rc,
};

use serde::Serialize;
use windows_sys::Win32::{
    Foundation::{FreeLibrary, GetLastError, HANDLE, HMODULE, HWND},
    System::LibraryLoader::{
        GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
        LoadLibraryExW,
    },
};

pub(super) const ATTITUDE_REQUEST: u32 = 1;
pub(super) const ATTITUDE_DEFINITION: u32 = 1;
// Request-side proxy; received dwObjectID may be the aircraft's runtime ID.
const OBJECT_ID_USER_AIRCRAFT: u32 = 0;
const OPEN_CONFIGINDEX_LOCAL: u32 = u32::MAX;
const UNUSED: u32 = u32::MAX;
const DATATYPE_FLOAT64: i32 = 4;
const PERIOD_SIM_FRAME: i32 = 3;
const E_FAIL: i32 = 0x8000_4005_u32 as i32;
const RECV_EXCEPTION: u32 = 1;
const RECV_OPEN: u32 = 2;
const RECV_QUIT: u32 = 3;
const RECV_EVENT: u32 = 4;
const RECV_EVENT_FILENAME: u32 = 6;
const RECV_SIMOBJECT_DATA: u32 = 8;
const RECV_SYSTEM_STATE: u32 = 15;

#[repr(C, packed)]
struct Recv {
    size: u32,
    version: u32,
    id: u32,
}

#[repr(C, packed)]
struct RecvException {
    header: Recv,
    exception: u32,
    send_id: u32,
    index: u32,
}

#[repr(C, packed)]
struct RecvOpen {
    header: Recv,
    application_name: [u8; 256],
    application_version: [u32; 4],
    simconnect_version: [u32; 4],
    reserved: [u32; 2],
}

#[repr(C, packed)]
struct RecvEvent {
    header: Recv,
    group_id: u32,
    event_id: u32,
    data: u32,
}

#[repr(C, packed)]
struct RecvFilename {
    event: RecvEvent,
    filename: [u8; 260],
    flags: u32,
}

#[repr(C, packed)]
struct RecvSystemState {
    header: Recv,
    request_id: u32,
    integer: u32,
    float: f32,
    string: [u8; 260],
}

#[repr(C, packed)]
struct RecvSimobjectData {
    header: Recv,
    request_id: u32,
    object_id: u32,
    define_id: u32,
    flags: u32,
    entry_number: u32,
    out_of: u32,
    define_count: u32,
    data: u32,
}

// The SDK declares extern "C" HRESULT. Windows x64 MSVC has one native ABI;
// C++ DWORD-based user IDs/flags are u32, and SDK enums are signed 32-bit ints.
type Open = unsafe extern "C" fn(*mut HANDLE, *const c_char, HWND, u32, HANDLE, u32) -> i32;
type Close = unsafe extern "C" fn(HANDLE) -> i32;
type GetNextDispatch = unsafe extern "C" fn(HANDLE, *mut *mut Recv, *mut u32) -> i32;
type AddToDataDefinition =
    unsafe extern "C" fn(HANDLE, u32, *const c_char, *const c_char, i32, f32, u32) -> i32;
type RequestDataOnSimObject =
    unsafe extern "C" fn(HANDLE, u32, u32, u32, i32, u32, u32, u32, u32) -> i32;
type SubscribeToSystemEvent = unsafe extern "C" fn(HANDLE, u32, *const c_char) -> i32;
type RequestSystemState = unsafe extern "C" fn(HANDLE, u32, *const c_char) -> i32;
type GetLastSentPacketId = unsafe extern "C" fn(HANDLE, *mut u32) -> i32;

struct Api {
    open: Open,
    close: Close,
    next: GetNextDispatch,
    add_definition: AddToDataDefinition,
    request_data: RequestDataOnSimObject,
    subscribe: SubscribeToSystemEvent,
    request_state: RequestSystemState,
    last_sent: GetLastSentPacketId,
}

#[derive(Debug, Serialize)]
pub(super) struct NativeError {
    pub operation: &'static str,
    pub hresult: Option<i32>,
    pub win32: Option<u32>,
    pub detail: String,
}

impl NativeError {
    fn detail(operation: &'static str, detail: impl Into<String>) -> Self {
        Self {
            operation,
            hresult: None,
            win32: None,
            detail: detail.into(),
        }
    }

    fn last_win32(operation: &'static str, detail: impl Into<String>) -> Self {
        // SAFETY: GetLastError has no preconditions and is read immediately.
        let win32 = unsafe { GetLastError() };
        Self {
            win32: Some(win32),
            ..Self::detail(operation, detail)
        }
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.detail)?;
        if let Some(hr) = self.hresult {
            write!(formatter, " (HRESULT 0x{:08X})", hr as u32)?;
        }
        if let Some(code) = self.win32 {
            write!(formatter, " (Windows error {code})")?;
        }
        Ok(())
    }
}

impl std::error::Error for NativeError {}

fn result(operation: &'static str, hr: i32) -> Result<(), NativeError> {
    if hr >= 0 {
        Ok(())
    } else {
        Err(NativeError {
            hresult: Some(hr),
            ..NativeError::detail(operation, "native API call failed")
        })
    }
}

/// The mapping, function pointers and all connections stay on their owner thread.
pub(super) struct Library {
    module: HMODULE,
    api: Api,
    retain_mapping: Cell<bool>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Library {
    pub(super) fn has_unconfirmed_close(&self) -> bool {
        self.retain_mapping.get()
    }

    pub fn load(path: &Path) -> Result<Self, NativeError> {
        if !path.is_absolute() {
            return Err(NativeError::detail(
                "LoadLibraryExW",
                "the DLL path must be absolute",
            ));
        }
        let mut path_wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if path_wide.contains(&0) {
            return Err(NativeError::detail(
                "LoadLibraryExW",
                "the DLL path contains NUL",
            ));
        }
        path_wide.push(0);
        // SAFETY: path_wide is NUL-terminated. Dependencies are restricted to
        // the selected DLL's directory and Windows system directory.
        let module = unsafe {
            LoadLibraryExW(
                path_wide.as_ptr(),
                ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        };
        if module.is_null() {
            return Err(NativeError::last_win32(
                "LoadLibraryExW",
                "could not load the selected DLL or a required dependency",
            ));
        }

        // Each conversion is between a GetProcAddress code pointer and the exact
        // hand-written signature inspected in the SDK header. No address escapes.
        let api = (|| {
            macro_rules! export {
                ($name:literal, $signature:ty) => {{
                    let address = unsafe { GetProcAddress(module, concat!($name, "\0").as_ptr()) }
                        .ok_or_else(|| {
                            NativeError::last_win32(
                                "GetProcAddress",
                                concat!("missing export ", $name),
                            )
                        })?;
                    unsafe {
                        std::mem::transmute::<unsafe extern "system" fn() -> isize, $signature>(
                            address,
                        )
                    }
                }};
            }
            Ok::<_, NativeError>(Api {
                open: export!("SimConnect_Open", Open),
                close: export!("SimConnect_Close", Close),
                next: export!("SimConnect_GetNextDispatch", GetNextDispatch),
                add_definition: export!("SimConnect_AddToDataDefinition", AddToDataDefinition),
                request_data: export!("SimConnect_RequestDataOnSimObject", RequestDataOnSimObject),
                subscribe: export!("SimConnect_SubscribeToSystemEvent", SubscribeToSystemEvent),
                request_state: export!("SimConnect_RequestSystemState", RequestSystemState),
                last_sent: export!("SimConnect_GetLastSentPacketID", GetLastSentPacketId),
            })
        })();
        match api {
            Ok(api) => Ok(Self {
                module,
                api,
                retain_mapping: Cell::new(false),
                _thread_bound: PhantomData,
            }),
            Err(error) => {
                // SAFETY: no connection was opened and no function pointer escapes.
                unsafe { FreeLibrary(module) };
                Err(error)
            }
        }
    }

    pub fn open(
        &self,
        notification: HANDLE,
        unit_probes: bool,
    ) -> Result<Connection<'_>, NativeError> {
        let mut handle = ptr::null_mut();
        // SAFETY: valid resolved function, writable handle output, static name.
        // The worker keeps its event alive until the returned connection closes.
        result("SimConnect_Open", unsafe {
            (self.api.open)(
                &mut handle,
                c"Artificial Horizon SimConnect".as_ptr(),
                ptr::null_mut(),
                0,
                notification,
                OPEN_CONFIGINDEX_LOCAL,
            )
        })?;
        if handle.is_null() {
            return Err(NativeError::detail(
                "SimConnect_Open",
                "success returned a null connection",
            ));
        }
        Ok(Connection {
            library: self,
            handle,
            dispatch_hresult: 0,
            unit_probes,
            _thread_bound: PhantomData,
        })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        if !self.retain_mapping.get() {
            // SAFETY: all connections borrow self, so none remain at this point.
            unsafe { FreeLibrary(self.module) };
        }
    }
}

pub(super) struct Connection<'library> {
    library: &'library Library,
    handle: HANDLE,
    dispatch_hresult: i32,
    // Fixed when opened, including if optional logging later fails.
    unit_probes: bool,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Connection<'_> {
    pub fn attitude_field_count(&self) -> usize {
        if self.unit_probes { 6 } else { 2 }
    }

    pub fn add_attitude_field(&mut self, index: usize) -> Result<(), NativeError> {
        let diagnostic_fields = [
            (c"PLANE PITCH DEGREES", c"radians"),
            (c"PLANE PITCH DEGREES", c"degrees"),
            (c"PLANE BANK DEGREES", c"radians"),
            (c"PLANE BANK DEGREES", c"degrees"),
            (c"PLANE PITCH DEGREES", c"radians"),
            (c"PLANE BANK DEGREES", c"radians"),
        ];
        let canonical_fields = [
            (c"PLANE PITCH DEGREES", c"radians"),
            (c"PLANE BANK DEGREES", c"radians"),
        ];
        let fields: &[_] = if self.unit_probes {
            &diagnostic_fields
        } else {
            &canonical_fields
        };
        let (name, units) = fields.get(index).ok_or_else(|| {
            NativeError::detail(
                "SimConnect_AddToDataDefinition",
                format!(
                    "attitude field index must be 0..{}",
                    self.attitude_field_count()
                ),
            )
        })?;
        // SAFETY: self owns the live handle, and names live through this call.
        result("SimConnect_AddToDataDefinition", unsafe {
            (self.library.api.add_definition)(
                self.handle,
                ATTITUDE_DEFINITION,
                name.as_ptr(),
                units.as_ptr(),
                DATATYPE_FLOAT64,
                0.0,
                UNUSED,
            )
        })
    }

    pub fn request_attitude(&mut self) -> Result<(), NativeError> {
        // SAFETY: resolved function and owned handle; flags=0 preserves equal samples.
        result("SimConnect_RequestDataOnSimObject", unsafe {
            (self.library.api.request_data)(
                self.handle,
                ATTITUDE_REQUEST,
                ATTITUDE_DEFINITION,
                OBJECT_ID_USER_AIRCRAFT,
                PERIOD_SIM_FRAME,
                0,
                0,
                0,
                0,
            )
        })
    }

    pub fn subscribe(&mut self, event_id: u32, name: &CStr) -> Result<(), NativeError> {
        // SAFETY: resolved function, owned handle and borrowed NUL-terminated name.
        result("SimConnect_SubscribeToSystemEvent", unsafe {
            (self.library.api.subscribe)(self.handle, event_id, name.as_ptr())
        })
    }

    pub fn request_state(&mut self, request_id: u32, name: &CStr) -> Result<(), NativeError> {
        // SAFETY: resolved function, owned handle and borrowed NUL-terminated name.
        result("SimConnect_RequestSystemState", unsafe {
            (self.library.api.request_state)(self.handle, request_id, name.as_ptr())
        })
    }

    pub fn last_sent_packet_id(&self) -> Result<u32, NativeError> {
        let mut id = 0;
        // SAFETY: resolved function, owned handle, writable DWORD output.
        result("SimConnect_GetLastSentPacketID", unsafe {
            (self.library.api.last_sent)(self.handle, &mut id)
        })?;
        Ok(id)
    }

    /// Raw status of the most recent dispatch, including an empty-queue E_FAIL.
    pub fn dispatch_hresult(&self) -> i32 {
        self.dispatch_hresult
    }

    pub fn next(&mut self) -> Result<Option<Packet>, NativeError> {
        let mut data = ptr::null_mut();
        let mut length = 0;
        // SAFETY: resolved function, owned handle and writable output parameters.
        let hr = unsafe { (self.library.api.next)(self.handle, &mut data, &mut length) };
        self.dispatch_hresult = hr;
        if hr == E_FAIL {
            // GetNextDispatch also uses E_FAIL for an empty queue. This is an
            // observation to aggregate, never standalone evidence of disconnect.
            return Ok(None);
        }
        result("SimConnect_GetNextDispatch", hr)?;
        // SAFETY: the SDK owns length bytes until the next dispatch. Decode now
        // into owned Rust values; no reference crosses the next native call.
        unsafe { decode_received(data, length, self.attitude_field_count()) }.map(Some)
    }

    pub fn close(mut self) -> Result<(), NativeError> {
        self.close_inner()
    }

    fn close_inner(&mut self) -> Result<(), NativeError> {
        if self.handle.is_null() {
            return Ok(());
        }
        let handle = std::mem::replace(&mut self.handle, ptr::null_mut());
        // SAFETY: this is the sole close attempt for the owned live handle.
        let closed = result("SimConnect_Close", unsafe {
            (self.library.api.close)(handle)
        });
        closed.map_err(|mut error| {
            self.library.retain_mapping.set(true);
            error
                .detail
                .push_str("; DLL mapping retained because connection closure is unconfirmed");
            error
        })
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Packet {
    pub id: u32,
    pub size: u32,
    pub version: u32,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_simobject_data: Option<RejectedSimobjectData>,
}

/// Header-only evidence for rejected data; the untrusted payload is never decoded.
#[derive(Debug, Serialize)]
pub(super) struct RejectedSimobjectData {
    reason: &'static str,
    request_id: u32,
    define_id: u32,
    object_id: u32,
    flags: u32,
    entry_number: u32,
    out_of: u32,
    define_count: u32,
    expected_request_id: u32,
    expected_define_id: u32,
    requested_object_proxy: u32,
    request_id_matches: bool,
    define_id_matches: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct UnitProbe {
    /// SDK degree values, pitch followed by bank; diagnostic only.
    pub degrees: [f64; 2],
    /// Repeated SDK radians, pitch followed by bank; diagnostic only.
    pub repeated_radians: [f64; 2],
}

#[derive(Debug, Serialize)]
pub(super) enum Message {
    Open {
        application_name: String,
        application_version: [u32; 4],
        simconnect_version: [u32; 4],
    },
    Quit,
    Exception {
        exception: u32,
        send_id: u32,
        index: u32,
    },
    Event {
        event_id: u32,
        data: u32,
    },
    Filename {
        event_id: u32,
        data: u32,
        filename: String,
        flags: u32,
    },
    SystemState {
        request_id: u32,
        integer: u32,
        float: f32,
        string: String,
    },
    Attitude {
        request_id: u32,
        define_id: u32,
        object_id: u32,
        entry_number: u32,
        out_of: u32,
        define_count: u32,
        /// Canonical pitch radians followed by bank radians.
        radians: [f64; 2],
        #[serde(skip_serializing_if = "Option::is_none")]
        unit_probe: Option<UnitProbe>,
    },
    Unknown,
}

fn malformed(detail: impl Into<String>) -> NativeError {
    NativeError::detail("decode receive packet", detail)
}

/// `data` must denote `length` readable bytes unless null or shorter than a header.
unsafe fn decode_received(
    data: *mut Recv,
    length: u32,
    attitude_field_count: usize,
) -> Result<Packet, NativeError> {
    if data.is_null() || length < size_of::<Recv>() as u32 {
        return Err(NativeError::detail(
            "SimConnect_GetNextDispatch",
            "missing or truncated receive header",
        ));
    }
    // SAFETY: caller guarantees readable bytes; alignment is irrelevant for u8.
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length as usize) };
    decode_with_attitude_count(bytes, attitude_field_count).map_err(|mut error| {
        error.detail.push_str(&format!(
            "; receive_id={}, receive_size={}, buffer_size={length}",
            word(bytes, 8).expect("validated receive header"),
            word(bytes, 0).expect("validated receive header"),
        ));
        error
    })
}

fn require(bytes: &[u8], minimum: usize) -> Result<(), NativeError> {
    if bytes.len() < minimum {
        Err(malformed(format!(
            "packet has {} bytes, requires {minimum}",
            bytes.len()
        )))
    } else {
        Ok(())
    }
}

fn word(bytes: &[u8], offset: usize) -> Result<u32, NativeError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| malformed("truncated DWORD"))?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("four-byte range"),
    ))
}

fn string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

#[cfg(test)]
fn decode(bytes: &[u8]) -> Result<Packet, NativeError> {
    decode_with_attitude_count(bytes, 2)
}

fn decode_with_attitude_count(
    bytes: &[u8],
    attitude_field_count: usize,
) -> Result<Packet, NativeError> {
    require(bytes, size_of::<Recv>())?;
    let size = word(bytes, 0)?;
    if size < size_of::<Recv>() as u32 || size as usize > bytes.len() {
        return Err(malformed(
            "receive header size exceeds buffer or is smaller than header",
        ));
    }
    let bytes = &bytes[..size as usize];
    let id = word(bytes, 8)?;
    let mut rejected_simobject_data = None;
    let message = match id {
        RECV_OPEN => {
            require(bytes, size_of::<RecvOpen>())?;
            Message::Open {
                application_name: string(&bytes[12..268]),
                application_version: [
                    word(bytes, 268)?,
                    word(bytes, 272)?,
                    word(bytes, 276)?,
                    word(bytes, 280)?,
                ],
                simconnect_version: [
                    word(bytes, 284)?,
                    word(bytes, 288)?,
                    word(bytes, 292)?,
                    word(bytes, 296)?,
                ],
            }
        }
        RECV_QUIT => Message::Quit,
        RECV_EXCEPTION => {
            require(bytes, size_of::<RecvException>())?;
            Message::Exception {
                exception: word(bytes, 12)?,
                send_id: word(bytes, 16)?,
                index: word(bytes, 20)?,
            }
        }
        RECV_EVENT => {
            require(bytes, size_of::<RecvEvent>())?;
            Message::Event {
                event_id: word(bytes, 16)?,
                data: word(bytes, 20)?,
            }
        }
        RECV_EVENT_FILENAME => {
            require(bytes, size_of::<RecvFilename>())?;
            Message::Filename {
                event_id: word(bytes, 16)?,
                data: word(bytes, 20)?,
                filename: string(&bytes[24..284]),
                flags: word(bytes, 284)?,
            }
        }
        RECV_SYSTEM_STATE => {
            require(bytes, size_of::<RecvSystemState>())?;
            Message::SystemState {
                request_id: word(bytes, 12)?,
                integer: word(bytes, 16)?,
                float: f32::from_bits(word(bytes, 20)?),
                string: string(&bytes[24..284]),
            }
        }
        RECV_SIMOBJECT_DATA => {
            // dwData is the first payload DWORD, not an extra 4-byte prefix.
            const DATA_OFFSET: usize = std::mem::offset_of!(RecvSimobjectData, data);
            require(bytes, DATA_OFFSET)?;
            let request_id = word(bytes, 12)?;
            let object_id = word(bytes, 16)?;
            let define_id = word(bytes, 20)?;
            // This connection owns one request sent for USER_AIRCRAFT (proxy 0).
            // Responses carry a runtime object ID; correlate by request/definition.
            if request_id != ATTITUDE_REQUEST || define_id != ATTITUDE_DEFINITION {
                rejected_simobject_data = Some(RejectedSimobjectData {
                    reason: "attitude_request_or_definition_mismatch",
                    request_id,
                    define_id,
                    object_id,
                    flags: word(bytes, 24)?,
                    entry_number: word(bytes, 28)?,
                    out_of: word(bytes, 32)?,
                    define_count: word(bytes, 36)?,
                    expected_request_id: ATTITUDE_REQUEST,
                    expected_define_id: ATTITUDE_DEFINITION,
                    requested_object_proxy: OBJECT_ID_USER_AIRCRAFT,
                    request_id_matches: request_id == ATTITUDE_REQUEST,
                    define_id_matches: define_id == ATTITUDE_DEFINITION,
                });
                Message::Unknown
            } else {
                let define_count = word(bytes, 36)?;
                let flags = word(bytes, 24)?;
                if !matches!(attitude_field_count, 2 | 6)
                    || define_count as usize != attitude_field_count
                    || flags != 0
                {
                    return Err(malformed(format!(
                        "attitude requires {attitude_field_count} untagged FLOAT64 data items; define_count={define_count}, flags={flags}",
                    )));
                }
                require(bytes, DATA_OFFSET + attitude_field_count * size_of::<f64>())?;
                let mut decoded_values = [0.0; 6];
                for (index, value) in decoded_values[..attitude_field_count]
                    .iter_mut()
                    .enumerate()
                {
                    let offset = DATA_OFFSET + index * 8;
                    *value = f64::from_le_bytes(
                        bytes[offset..offset + 8]
                            .try_into()
                            .expect("eight-byte range"),
                    );
                }
                let (radians, unit_probe) = if attitude_field_count == 6 {
                    (
                        [decoded_values[0], decoded_values[2]],
                        Some(UnitProbe {
                            degrees: [decoded_values[1], decoded_values[3]],
                            repeated_radians: [decoded_values[4], decoded_values[5]],
                        }),
                    )
                } else {
                    ([decoded_values[0], decoded_values[1]], None)
                };
                Message::Attitude {
                    request_id,
                    define_id,
                    object_id,
                    entry_number: word(bytes, 28)?,
                    out_of: word(bytes, 32)?,
                    define_count,
                    radians,
                    unit_probe,
                }
            }
        }
        _ => Message::Unknown,
    };
    Ok(Packet {
        id,
        size,
        version: word(bytes, 4)?,
        message,
        rejected_simobject_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn native_owners_cannot_cross_threads_or_be_shared() {
        // Type inference becomes ambiguous if the tested type implements the
        // forbidden trait. This keeps the assertion independent of dependencies.
        macro_rules! assert_not_impl {
            ($owner:ty, $forbidden:path) => {{
                trait AmbiguousIfImpl<Marker> {
                    fn check() {}
                }
                impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
                impl<T: ?Sized + $forbidden> AmbiguousIfImpl<u8> for T {}
                let _ = <$owner as AmbiguousIfImpl<_>>::check;
            }};
        }
        assert_not_impl!(Library, Send);
        assert_not_impl!(Library, Sync);
        assert_not_impl!(Connection<'static>, Send);
        assert_not_impl!(Connection<'static>, Sync);
    }

    #[test]
    fn dispatch_pointer_invariants_are_checked_before_creating_a_slice() {
        for length in [0, 11, 12, u32::MAX] {
            // SAFETY: a null pointer is rejected before accessing any bytes.
            assert!(unsafe { decode_received(ptr::null_mut(), length, 2) }.is_err());
        }
        let mut bytes = packet(RECV_QUIT, 12);
        for length in 0..12 {
            // SAFETY: the helper rejects short lengths before accessing bytes.
            assert!(unsafe { decode_received(bytes.as_mut_ptr().cast(), length, 2) }.is_err());
        }
        // SAFETY: bytes contains a complete readable receive header.
        assert!(matches!(
            unsafe { decode_received(bytes.as_mut_ptr().cast(), 12, 2) }
                .unwrap()
                .message,
            Message::Quit
        ));
    }

    thread_local! {
        static CLOSE_RESULT: Cell<i32> = const { Cell::new(0) };
        static CLOSE_CALLS: Cell<u32> = const { Cell::new(0) };
        static NEXT_RESULT: Cell<i32> = const { Cell::new(E_FAIL) };
        static NEXT_PACKET: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
        static ADD_FIELDS: std::cell::RefCell<Vec<(String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    unsafe extern "C" fn mock_close(_: HANDLE) -> i32 {
        CLOSE_CALLS.with(|calls| calls.set(calls.get() + 1));
        CLOSE_RESULT.with(Cell::get)
    }
    unsafe extern "C" fn mock_open(
        handle: *mut HANDLE,
        _: *const c_char,
        _: HWND,
        _: u32,
        _: HANDLE,
        _: u32,
    ) -> i32 {
        // SAFETY: Library::open supplies its writable handle output.
        unsafe { *handle = ptr::without_provenance_mut(1) };
        0
    }
    unsafe extern "C" fn mock_next(_: HANDLE, data: *mut *mut Recv, length: *mut u32) -> i32 {
        NEXT_PACKET.with(|packet| {
            let mut packet = packet.borrow_mut();
            if !packet.is_empty() {
                // SAFETY: Connection::next supplies valid writable outputs;
                // the thread-local buffer is retained until after decoding.
                unsafe {
                    *data = packet.as_mut_ptr().cast();
                    *length = packet.len() as u32;
                }
            }
        });
        NEXT_RESULT.with(Cell::get)
    }
    unsafe extern "C" fn mock_add(
        _: HANDLE,
        definition: u32,
        name: *const c_char,
        units: *const c_char,
        datatype: i32,
        epsilon: f32,
        datum_id: u32,
    ) -> i32 {
        assert_eq!(definition, ATTITUDE_DEFINITION);
        assert_eq!(datatype, DATATYPE_FLOAT64);
        assert_eq!(epsilon, 0.0);
        assert_eq!(datum_id, UNUSED);
        // SAFETY: add_attitude_field supplies static NUL-terminated strings.
        let (name, units) = unsafe { (CStr::from_ptr(name), CStr::from_ptr(units)) };
        ADD_FIELDS.with(|fields| {
            fields.borrow_mut().push((
                name.to_str().unwrap().to_owned(),
                units.to_str().unwrap().to_owned(),
            ));
        });
        0
    }
    unsafe extern "C" fn mock_request(
        _: HANDLE,
        _: u32,
        _: u32,
        _: u32,
        _: i32,
        _: u32,
        _: u32,
        _: u32,
        _: u32,
    ) -> i32 {
        0
    }
    unsafe extern "C" fn mock_subscribe(_: HANDLE, _: u32, _: *const c_char) -> i32 {
        0
    }
    unsafe extern "C" fn mock_state(_: HANDLE, _: u32, _: *const c_char) -> i32 {
        0
    }
    unsafe extern "C" fn mock_last(_: HANDLE, _: *mut u32) -> i32 {
        0
    }

    fn mock_library() -> Library {
        CLOSE_CALLS.with(|value| value.set(0));
        CLOSE_RESULT.with(|value| value.set(0));
        NEXT_RESULT.with(|value| value.set(E_FAIL));
        NEXT_PACKET.with(|packet| packet.borrow_mut().clear());
        ADD_FIELDS.with(|fields| fields.borrow_mut().clear());
        Library {
            module: ptr::null_mut(),
            api: Api {
                open: mock_open,
                close: mock_close,
                next: mock_next,
                add_definition: mock_add,
                request_data: mock_request,
                subscribe: mock_subscribe,
                request_state: mock_state,
                last_sent: mock_last,
            },
            // Mock owns no module mapping. Suppress FreeLibrary(null).
            retain_mapping: Cell::new(true),
            _thread_bound: PhantomData,
        }
    }

    fn mock_connection(library: &Library) -> Connection<'_> {
        Connection {
            library,
            handle: ptr::without_provenance_mut(1),
            dispatch_hresult: 0,
            unit_probes: false,
            _thread_bound: PhantomData,
        }
    }

    #[test]
    fn explicit_close_and_drop_each_close_once() {
        let library = mock_library();
        mock_connection(&library).close().unwrap();
        assert_eq!(CLOSE_CALLS.with(Cell::get), 1);
        drop(mock_connection(&library));
        assert_eq!(CLOSE_CALLS.with(Cell::get), 2);
    }

    #[test]
    fn failed_close_preserves_error_and_retains_mapping() {
        let library = mock_library();
        library.retain_mapping.set(false);
        CLOSE_RESULT.with(|value| value.set(E_FAIL));
        let error = mock_connection(&library).close().unwrap_err();
        assert_eq!(error.operation, "SimConnect_Close");
        assert_eq!(error.hresult, Some(E_FAIL));
        assert!(library.retain_mapping.get());
        assert_eq!(CLOSE_CALLS.with(Cell::get), 1);
    }

    #[test]
    fn empty_dispatch_is_observable_without_asserting_disconnect() {
        let library = mock_library();
        let mut connection = mock_connection(&library);
        assert!(connection.next().unwrap().is_none());
        assert_eq!(connection.dispatch_hresult(), E_FAIL);
        NEXT_RESULT.with(|value| value.set(0x8000_0008_u32 as i32));
        assert_eq!(
            connection.next().unwrap_err().hresult,
            Some(0x8000_0008_u32 as i32)
        );
        // A successful dispatch without a buffer remains a controlled error.
        NEXT_RESULT.with(|value| value.set(0));
        assert!(connection.next().is_err());
    }

    #[test]
    fn connection_mode_fixes_field_order_count_and_index_bounds() {
        for unit_probes in [false, true] {
            let library = mock_library();
            let mut connection = library.open(ptr::null_mut(), unit_probes).unwrap();
            let count = if unit_probes { 6 } else { 2 };
            assert_eq!(connection.attitude_field_count(), count);
            for index in 0..count {
                connection.add_attitude_field(index).unwrap();
            }
            for index in [count, usize::MAX] {
                let error = connection.add_attitude_field(index).unwrap_err();
                assert_eq!(error.operation, "SimConnect_AddToDataDefinition");
            }
            let diagnostic_fields = [
                ("PLANE PITCH DEGREES", "radians"),
                ("PLANE PITCH DEGREES", "degrees"),
                ("PLANE BANK DEGREES", "radians"),
                ("PLANE BANK DEGREES", "degrees"),
                ("PLANE PITCH DEGREES", "radians"),
                ("PLANE BANK DEGREES", "radians"),
            ];
            let canonical_fields = [
                ("PLANE PITCH DEGREES", "radians"),
                ("PLANE BANK DEGREES", "radians"),
            ];
            let expected: &[_] = if unit_probes {
                &diagnostic_fields
            } else {
                &canonical_fields
            };
            ADD_FIELDS.with(|fields| {
                let fields = fields.borrow();
                let borrowed: Vec<_> = fields
                    .iter()
                    .map(|(name, units)| (name.as_str(), units.as_str()))
                    .collect();
                assert_eq!(borrowed, expected);
            });
        }
    }

    #[test]
    fn dispatch_decoding_uses_the_mode_fixed_at_connection_open() {
        for unit_probes in [false, true] {
            let library = mock_library();
            let mut connection = library.open(ptr::null_mut(), unit_probes).unwrap();
            NEXT_RESULT.with(|value| value.set(0));
            let matching = if unit_probes {
                attitude_with_repeats([0.25, -0.5])
            } else {
                attitude()
            };
            NEXT_PACKET.with(|packet| *packet.borrow_mut() = matching);
            assert!(matches!(
                connection.next().unwrap().unwrap().message,
                Message::Attitude { radians: [0.25, -0.5], unit_probe, .. }
                    if unit_probe.is_some() == unit_probes
            ));
            let mismatch = if unit_probes {
                attitude()
            } else {
                attitude_with_repeats([0.25, -0.5])
            };
            NEXT_PACKET.with(|packet| *packet.borrow_mut() = mismatch);
            assert!(connection.next().is_err());
        }
    }

    #[test]
    fn abi_matches_inspected_sdk_header_on_windows_x64() {
        assert_eq!(size_of::<HANDLE>(), 8);
        assert_eq!(size_of::<HWND>(), 8);
        assert_eq!(size_of::<Open>(), 8);
        assert_eq!(size_of::<Recv>(), 12);
        assert_eq!(align_of::<Recv>(), 1);
        assert_eq!(offset_of!(Recv, id), 8);
        assert_eq!(size_of::<RecvException>(), 24);
        assert_eq!(align_of::<RecvException>(), 1);
        assert_eq!(offset_of!(RecvException, send_id), 16);
        assert_eq!(size_of::<RecvOpen>(), 308);
        assert_eq!(align_of::<RecvOpen>(), 1);
        assert_eq!(offset_of!(RecvOpen, application_version), 268);
        assert_eq!(offset_of!(RecvOpen, simconnect_version), 284);
        assert_eq!(offset_of!(RecvOpen, reserved), 300);
        assert_eq!(size_of::<RecvEvent>(), 24);
        assert_eq!(align_of::<RecvEvent>(), 1);
        assert_eq!(offset_of!(RecvEvent, event_id), 16);
        assert_eq!(size_of::<RecvFilename>(), 288);
        assert_eq!(align_of::<RecvFilename>(), 1);
        assert_eq!(offset_of!(RecvFilename, flags), 284);
        assert_eq!(size_of::<RecvSystemState>(), 284);
        assert_eq!(align_of::<RecvSystemState>(), 1);
        assert_eq!(offset_of!(RecvSystemState, float), 20);
        assert_eq!(offset_of!(RecvSystemState, string), 24);
        assert_eq!(size_of::<RecvSimobjectData>(), 44);
        assert_eq!(align_of::<RecvSimobjectData>(), 1);
        assert_eq!(offset_of!(RecvSimobjectData, data), 40);
        assert_eq!(offset_of!(RecvSimobjectData, define_count), 36);
        assert_eq!(DATATYPE_FLOAT64, 4);
        assert_eq!(PERIOD_SIM_FRAME, 3);
        assert_eq!(OPEN_CONFIGINDEX_LOCAL, 0xffff_ffff);
        assert_eq!(UNUSED, 0xffff_ffff);
        assert_eq!(OBJECT_ID_USER_AIRCRAFT, 0);
        assert_eq!(
            [
                RECV_EXCEPTION,
                RECV_OPEN,
                RECV_QUIT,
                RECV_EVENT,
                RECV_EVENT_FILENAME,
                RECV_SIMOBJECT_DATA,
                RECV_SYSTEM_STATE
            ],
            [1, 2, 3, 4, 6, 8, 15]
        );
    }

    fn packet(id: u32, size: usize) -> Vec<u8> {
        let mut bytes = vec![0; size];
        bytes[..4].copy_from_slice(&(size as u32).to_le_bytes());
        bytes[4..8].copy_from_slice(&7_u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&id.to_le_bytes());
        bytes
    }

    fn attitude() -> Vec<u8> {
        let mut bytes = packet(RECV_SIMOBJECT_DATA, 56);
        bytes[12..16].copy_from_slice(&ATTITUDE_REQUEST.to_le_bytes());
        bytes[20..24].copy_from_slice(&ATTITUDE_DEFINITION.to_le_bytes());
        bytes[28..32].copy_from_slice(&1_u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&1_u32.to_le_bytes());
        bytes[36..40].copy_from_slice(&2_u32.to_le_bytes());
        for (index, value) in [0.25_f64, -0.5].iter().enumerate() {
            bytes[40 + index * 8..48 + index * 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    fn attitude_with_repeats(repeated_radians: [f64; 2]) -> Vec<u8> {
        let mut bytes = attitude();
        bytes.resize(88, 0);
        bytes[..4].copy_from_slice(&88_u32.to_le_bytes());
        bytes[36..40].copy_from_slice(&6_u32.to_le_bytes());
        let values = [
            0.25_f64,
            14.32,
            -0.5,
            -28.64,
            repeated_radians[0],
            repeated_radians[1],
        ];
        for (index, value) in values.iter().enumerate() {
            bytes[40 + index * 8..48 + index * 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn copies_unaligned_payload_from_offset_40() {
        let bytes = attitude();
        let mut unaligned = vec![0];
        unaligned.extend_from_slice(&bytes);
        let decoded = decode(&unaligned[1..]).unwrap();
        assert_eq!((decoded.id, decoded.size, decoded.version), (8, 56, 7));
        match decoded.message {
            Message::Attitude {
                radians,
                entry_number,
                out_of,
                unit_probe,
                ..
            } => {
                assert_eq!(radians, [0.25, -0.5]);
                assert_eq!((entry_number, out_of), (1, 1));
                assert!(unit_probe.is_none());
            }
            message => panic!("unexpected {message:?}"),
        }
    }

    #[test]
    fn canonical_two_field_profile_preserves_raw_radian_bits() {
        for radians in [
            [0.250_123_456_789_f64, -0.500_987_654_321],
            [f64::INFINITY, f64::NEG_INFINITY],
            [f64::from_bits(0x7ff8_1234_5678_9abc), -0.0],
        ] {
            let mut bytes = attitude();
            for (index, value) in radians.iter().enumerate() {
                bytes[40 + index * 8..48 + index * 8].copy_from_slice(&value.to_le_bytes());
            }
            match decode(&bytes).unwrap().message {
                Message::Attitude {
                    radians: actual,
                    unit_probe: None,
                    define_count: 2,
                    ..
                } => {
                    assert_eq!(actual.map(f64::to_bits), radians.map(f64::to_bits));
                }
                message => panic!("unexpected {message:?}"),
            }
        }
    }

    #[test]
    fn repeated_radians_preserve_six_field_offset_order_and_bits() {
        let probes = [
            [0.250_123_456_789_f64, -0.500_987_654_321],
            [f64::INFINITY, f64::NEG_INFINITY],
            [f64::from_bits(0x7ff8_1234_5678_9abc), -0.0],
        ];
        for repeated in probes {
            let mut bytes = attitude_with_repeats(repeated);
            // All six fields must remain raw copies, including non-finite bits.
            let expected = [repeated[0], repeated[1], -0.5, -28.64];
            for (index, value) in expected.iter().enumerate() {
                bytes[40 + index * 8..48 + index * 8].copy_from_slice(&value.to_le_bytes());
            }
            let mut unaligned = vec![0];
            unaligned.extend_from_slice(&bytes);
            let decoded = decode_with_attitude_count(&unaligned[1..], 6).unwrap();
            assert_eq!((decoded.id, decoded.size, decoded.version), (8, 88, 7));
            match decoded.message {
                Message::Attitude {
                    radians,
                    unit_probe:
                        Some(UnitProbe {
                            degrees,
                            repeated_radians: actual,
                        }),
                    define_count,
                    ..
                } => {
                    assert_eq!(define_count, 6);
                    assert_eq!(
                        radians.map(f64::to_bits),
                        [expected[0], expected[2]].map(f64::to_bits)
                    );
                    assert_eq!(
                        degrees.map(f64::to_bits),
                        [expected[1], expected[3]].map(f64::to_bits)
                    );
                    assert_eq!(actual.map(f64::to_bits), repeated.map(f64::to_bits));
                }
                message => panic!("unexpected {message:?}"),
            }
        }
        let two = serde_json::to_value(decode(&attitude()).unwrap()).unwrap();
        assert!(two["message"]["Attitude"].get("unit_probe").is_none());
        let six = serde_json::to_value(
            decode_with_attitude_count(&attitude_with_repeats([0.25, -0.5]), 6).unwrap(),
        )
        .unwrap();
        assert_eq!(
            six["message"]["Attitude"]["unit_probe"]["repeated_radians"],
            serde_json::json!([0.25, -0.5])
        );
        assert_eq!(
            six["message"]["Attitude"]["radians"],
            serde_json::json!([0.25, -0.5])
        );
        assert_eq!(
            six["message"]["Attitude"]["unit_probe"]["degrees"],
            serde_json::json!([14.32, -28.64])
        );
    }

    #[test]
    fn rejects_every_truncated_six_field_payload_and_both_mode_mismatches() {
        let bytes = attitude_with_repeats([0.25, -0.5]);
        for length in 0..bytes.len() {
            assert!(
                decode_with_attitude_count(&bytes[..length], 6).is_err(),
                "buffer length={length}"
            );
            if length >= 12 {
                let mut short = bytes[..length].to_vec();
                short[..4].copy_from_slice(&(length as u32).to_le_bytes());
                assert!(
                    decode_with_attitude_count(&short, 6).is_err(),
                    "declared length={length}"
                );
            }
        }
        assert!(decode_with_attitude_count(&bytes, 2).is_err());
        assert!(decode_with_attitude_count(&attitude(), 6).is_err());
        for count in [0, 2, 3, 4, 5, 7, u32::MAX] {
            let mut wrong_count = bytes.clone();
            wrong_count[36..40].copy_from_slice(&count.to_le_bytes());
            assert!(decode_with_attitude_count(&wrong_count, 6).is_err());
        }
        let mut tagged = bytes.clone();
        tagged[24..28].copy_from_slice(&2_u32.to_le_bytes());
        assert!(decode_with_attitude_count(&tagged, 6).is_err());
    }

    #[test]
    fn six_field_mode_keeps_request_correlation_and_unknown_packet_behavior() {
        let mut bytes = attitude_with_repeats([0.25, -0.5]);
        for runtime_id in [0_u32, 540_672, 540_673, u32::MAX] {
            bytes[16..20].copy_from_slice(&runtime_id.to_le_bytes());
            assert!(matches!(
                decode_with_attitude_count(&bytes, 6).unwrap().message,
                Message::Attitude { object_id, .. } if object_id == runtime_id
            ));
        }
        for offset in [12, 20] {
            let mut unknown = bytes[..40].to_vec();
            unknown[..4].copy_from_slice(&40_u32.to_le_bytes());
            unknown[offset..offset + 4].copy_from_slice(&99_u32.to_le_bytes());
            let decoded = decode_with_attitude_count(&unknown, 6).unwrap();
            assert!(matches!(decoded.message, Message::Unknown));
            assert!(decoded.rejected_simobject_data.is_some());
        }
        assert!(matches!(
            decode_with_attitude_count(&packet(0xf000, 12), 6)
                .unwrap()
                .message,
            Message::Unknown
        ));
    }

    #[test]
    fn rejects_every_truncated_attitude_without_panicking() {
        let bytes = attitude();
        for length in 0..bytes.len() {
            assert!(decode(&bytes[..length]).is_err(), "length={length}");
            if length >= 12 {
                let mut short = bytes[..length].to_vec();
                short[..4].copy_from_slice(&(length as u32).to_le_bytes());
                assert!(decode(&short).is_err(), "declared length={length}");
            }
        }
    }

    #[test]
    fn validates_attitude_identifiers_count_and_flags_before_payload() {
        for offset in [12, 20] {
            let mut bytes = attitude()[..40].to_vec();
            bytes[..4].copy_from_slice(&40_u32.to_le_bytes());
            bytes[offset..offset + 4].copy_from_slice(&99_u32.to_le_bytes());
            let decoded = decode(&bytes).unwrap();
            assert!(matches!(decoded.message, Message::Unknown));
            let diagnostic = serde_json::to_value(&decoded).unwrap();
            let rejected = &diagnostic["rejected_simobject_data"];
            assert_eq!(
                rejected["reason"],
                "attitude_request_or_definition_mismatch"
            );
            assert_eq!(rejected["expected_request_id"], 1);
            assert_eq!(rejected["expected_define_id"], 1);
            assert_eq!(rejected["requested_object_proxy"], 0);
            assert_eq!(rejected["request_id"], if offset == 12 { 99 } else { 1 });
            assert_eq!(rejected["define_id"], if offset == 20 { 99 } else { 1 });
            assert_eq!(rejected["request_id_matches"], offset != 12);
            assert_eq!(rejected["define_id_matches"], offset != 20);
            assert!(rejected.get("object_id_matches").is_none());
        }
        for (offset, value) in [
            (36, 0_u32),
            (36, 1),
            (36, 3),
            (36, 4),
            (36, 5),
            (36, 6),
            (36, u32::MAX),
            (24, 2),
        ] {
            let mut bytes = attitude();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(decode(&bytes).is_err());
        }
    }

    #[test]
    fn user_aircraft_request_accepts_the_runtime_object_id_observed_in_live_logs() {
        // rejected-ids.jsonl records 37..1438: matching request/definition 1,
        // runtime object 540672 and one returned object.
        // Payload values here are synthetic; the old log did not retain them.
        for runtime_id in [0_u32, 540_672, 540_673, u32::MAX] {
            let mut bytes = attitude();
            bytes[16..20].copy_from_slice(&runtime_id.to_le_bytes());
            let decoded = decode(&bytes).unwrap();
            match decoded.message {
                Message::Attitude {
                    request_id,
                    define_id,
                    object_id,
                    radians,
                    ..
                } => {
                    assert_eq!((request_id, define_id, object_id), (1, 1, runtime_id));
                    assert_eq!(radians, [0.25, -0.5]);
                }
                message => panic!("user-aircraft response was rejected: {message:?}"),
            }
            assert!(decoded.rejected_simobject_data.is_none());
            // A runtime object ID must not bypass payload/shape validation.
            for (offset, value) in [(24, 2_u32), (36, 3)] {
                let mut malformed = bytes.clone();
                malformed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                assert!(decode(&malformed).is_err());
            }
            bytes.truncate(55);
            bytes[..4].copy_from_slice(&55_u32.to_le_bytes());
            assert!(decode(&bytes).is_err());
        }
    }

    #[test]
    fn unknown_request_retains_object_identity_without_decoding_its_payload() {
        let mut bytes = attitude();
        bytes[12..16].copy_from_slice(&99_u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&42_u32.to_le_bytes());
        let decoded = decode(&bytes).unwrap();
        assert!(matches!(decoded.message, Message::Unknown));
        assert_eq!((decoded.id, decoded.size, decoded.version), (8, 56, 7));
        let diagnostic = serde_json::to_value(&decoded).unwrap();
        let rejected = &diagnostic["rejected_simobject_data"];
        assert_eq!(diagnostic["message"], "Unknown");
        assert_eq!(rejected["object_id"], 42);
        assert_eq!(rejected["requested_object_proxy"], 0);
        assert!(rejected.get("object_id_matches").is_none());
        assert_eq!(rejected["request_id_matches"], false);
        assert_eq!(rejected["define_id_matches"], true);
        assert_eq!(rejected["flags"], 0);
        assert_eq!(rejected["define_count"], 2);
        assert_eq!(rejected["entry_number"], 1);
        assert_eq!(rejected["out_of"], 1);

        // Reject the unknown request before inspecting its payload or attitude shape.
        bytes.truncate(40);
        bytes[..4].copy_from_slice(&40_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&2_u32.to_le_bytes());
        bytes[36..40].copy_from_slice(&99_u32.to_le_bytes());
        let decoded = decode(&bytes).unwrap();
        assert!(matches!(decoded.message, Message::Unknown));
        let rejected = serde_json::to_value(decoded).unwrap()["rejected_simobject_data"].clone();
        assert_eq!(rejected["flags"], 2);
        assert_eq!(rejected["define_count"], 99);
    }

    #[test]
    fn rejection_diagnostics_preserve_request_and_definition_mismatches() {
        let mut bytes = attitude();
        for (offset, value) in [(12, 123_u32), (16, 456), (20, 789)] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        let value = serde_json::to_value(decode(&bytes).unwrap()).unwrap();
        let rejected = &value["rejected_simobject_data"];
        assert_eq!(rejected["request_id"], 123);
        assert_eq!(rejected["object_id"], 456);
        assert_eq!(rejected["define_id"], 789);
        for key in ["request_id_matches", "define_id_matches"] {
            assert_eq!(rejected[key], false);
        }
        assert_eq!(rejected["requested_object_proxy"], 0);
        assert!(rejected.get("expected_object_id").is_none());
        // Matching attitudes and unsupported receive IDs must not claim rejection.
        for bytes in [attitude(), packet(0xf000, 12)] {
            let value = serde_json::to_value(decode(&bytes).unwrap()).unwrap();
            assert!(value.get("rejected_simobject_data").is_none());
        }
    }

    #[test]
    fn unknown_ids_and_extended_packets_preserve_header() {
        let decoded = decode(&packet(0xf000, 12)).unwrap();
        assert!(matches!(decoded.message, Message::Unknown));
        assert_eq!(decoded.id, 0xf000);
        assert!(matches!(
            decode(&packet(RECV_QUIT, 28)).unwrap().message,
            Message::Quit
        ));
        let mut invalid_size = packet(RECV_QUIT, 12);
        invalid_size[..4].copy_from_slice(&11_u32.to_le_bytes());
        assert!(decode(&invalid_size).is_err());
    }

    #[test]
    fn decodes_owned_strings_versions_exceptions_and_lifecycle_events() {
        let mut open = packet(RECV_OPEN, 308);
        open[12..16].copy_from_slice(b"MSFS");
        open[268..272].copy_from_slice(&2024_u32.to_le_bytes());
        open[284..288].copy_from_slice(&12_u32.to_le_bytes());
        match decode(&open).unwrap().message {
            Message::Open {
                application_name,
                application_version,
                simconnect_version,
            } => {
                assert_eq!(application_name, "MSFS");
                assert_eq!(application_version, [2024, 0, 0, 0]);
                assert_eq!(simconnect_version, [12, 0, 0, 0]);
            }
            other => panic!("unexpected {other:?}"),
        }
        let mut event = packet(RECV_EVENT, 24);
        event[16..20].copy_from_slice(&42_u32.to_le_bytes());
        event[20..24].copy_from_slice(&4_u32.to_le_bytes());
        assert!(matches!(
            decode(&event).unwrap().message,
            Message::Event {
                event_id: 42,
                data: 4
            }
        ));
        let mut filename = packet(RECV_EVENT_FILENAME, 288);
        filename[24..28].copy_from_slice(b"test");
        filename[284..288].copy_from_slice(&3_u32.to_le_bytes());
        assert!(
            matches!(decode(&filename).unwrap().message,Message::Filename {filename,flags:3,..} if filename=="test")
        );
        let mut state = packet(RECV_SYSTEM_STATE, 284);
        state[12..16].copy_from_slice(&20_u32.to_le_bytes());
        state[16..20].copy_from_slice(&1_u32.to_le_bytes());
        state[20..24].copy_from_slice(&1.25_f32.to_le_bytes());
        state[24..28].copy_from_slice(b"sim\0");
        assert!(
            matches!(decode(&state).unwrap().message, Message::SystemState {request_id:20,integer:1,float:1.25,string} if string=="sim")
        );
        let mut exception = packet(RECV_EXCEPTION, 24);
        exception[12..16].copy_from_slice(&8_u32.to_le_bytes());
        exception[16..20].copy_from_slice(&100_u32.to_le_bytes());
        exception[20..24].copy_from_slice(&2_u32.to_le_bytes());
        assert!(matches!(
            decode(&exception).unwrap().message,
            Message::Exception {
                exception: 8,
                send_id: 100,
                index: 2
            }
        ));
    }

    #[test]
    fn malformed_fixed_packets_and_unterminated_strings_are_bounded() {
        for (id, size) in [
            (RECV_OPEN, 308),
            (RECV_EXCEPTION, 24),
            (RECV_EVENT, 24),
            (RECV_EVENT_FILENAME, 288),
            (RECV_SYSTEM_STATE, 284),
        ] {
            assert!(decode(&packet(id, size - 1)).is_err());
        }
        let mut bytes = packet(RECV_SYSTEM_STATE, 284);
        bytes[24..].fill(b'x');
        assert!(
            matches!(decode(&bytes).unwrap().message, Message::SystemState {string,..} if string.len()==260)
        );
    }

    #[test]
    fn loader_rejects_relative_missing_and_embedded_nul_paths() {
        assert_eq!(
            Library::load(Path::new("SimConnect.dll"))
                .err()
                .unwrap()
                .operation,
            "LoadLibraryExW"
        );
        let missing =
            std::env::temp_dir().join(format!("missing-simconnect-{}.dll", std::process::id()));
        let error = Library::load(&missing).err().unwrap();
        assert_eq!(error.operation, "LoadLibraryExW");
        assert!(error.win32.is_some());
        assert!(Library::load(Path::new("C:\\invalid\0.dll")).is_err());
    }

    #[test]
    fn loader_reports_missing_export_from_system_dll() {
        let windows = std::env::var_os("SystemRoot").expect("Windows system directory");
        let dll = Path::new(&windows).join("System32").join("kernel32.dll");
        let error = Library::load(&dll).err().unwrap();
        assert_eq!(error.operation, "GetProcAddress");
        assert_eq!(error.detail, "missing export SimConnect_Open");
        assert!(error.win32.is_some());
    }
}
