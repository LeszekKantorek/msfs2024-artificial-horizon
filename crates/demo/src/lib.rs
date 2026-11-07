//! Project-owned implementation of the SimConnect subset used by the application.
mod scenario;
use scenario::{Event, Scenario};
use std::{
    collections::{HashMap, VecDeque},
    ffi::{CStr, OsString, c_char, c_void},
    io::{Read, Write},
    os::windows::ffi::{OsStrExt, OsStringExt},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    ptr,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Threading::SetEvent;
const FAIL: i32 = 0x80004005u32 as i32;
const DEMO_CONFIGURATION_ERROR: i32 = 0xA004_0001u32 as i32;
const MAX_QUEUE: usize = 8192;
static GLOBAL: OnceLock<Mutex<Global>> = OnceLock::new();
struct Global {
    scenario: Option<Scenario>,
    origin: Option<Instant>,
    lifecycle: Option<Life>,
    active: bool,
    fault_counts: Vec<u64>,
}
#[derive(Clone)]
struct Life {
    sim: bool,
    pause: u32,
    flight: Option<String>,
    through: u64,
    unavailable_until: u64,
}
impl Global {
    fn new() -> Self {
        Self {
            scenario: None,
            origin: None,
            lifecycle: None,
            active: false,
            fault_counts: Vec::new(),
        }
    }
    fn fault(&mut self, operation: &str, name: Option<&str>, unit: Option<&str>) -> Option<i32> {
        let rules = &self.scenario.as_ref()?.faults;
        let mut result = None;
        for (rule, count) in rules.iter().zip(&mut self.fault_counts) {
            if rule.operation != operation {
                continue;
            }
            if let Some(selector) = &rule.selector
                && (selector.simvar.as_deref().is_some_and(|s| Some(s) != name)
                    || selector.unit.as_deref().is_some_and(|s| Some(s) != unit))
            {
                continue;
            }
            *count = count.saturating_add(1);
            if *count == rule.occurrence && result.is_none() {
                result = Some(rule.hresult);
            }
        }
        result
    }
}
fn global() -> &'static Mutex<Global> {
    GLOBAL.get_or_init(|| Mutex::new(Global::new()))
}
struct State {
    queue: VecDeque<(Vec<u8>, u32)>,
    retained: Vec<u8>,
    definitions: HashMap<u32, Vec<(String, String)>>,
    requests: Vec<(u32, u32)>,
    subscriptions: HashMap<String, u32>,
    last_id: u32,
    life: Life,
    overflow: bool,
}
struct Connection {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    notification: usize,
}
fn guard(f: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(FAIL)
}
unsafe fn conn<'a>(h: *mut c_void) -> Option<&'a mut Connection> {
    unsafe { (h as *mut Connection).as_mut() }
}
unsafe fn string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}
fn packet(id: u32, body: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(12 + body.len());
    p.extend_from_slice(&((12 + body.len()) as u32).to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&id.to_le_bytes());
    p.extend_from_slice(body);
    p
}
fn u32s(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn send(s: &mut State, event: usize, p: Vec<u8>) {
    let len = p.len() as u32;
    send_raw(s, event, p, len);
}
fn send_raw(s: &mut State, event: usize, mut p: Vec<u8>, len: u32) {
    if s.queue.len() >= MAX_QUEUE {
        s.overflow = true;
        return;
    }
    // Keep even a zero-length result backed by a readable allocation.
    if p.is_empty() {
        p.push(0);
    }
    s.queue.push_back((p, len));
    if event != 0 {
        unsafe {
            SetEvent(event as *mut c_void);
        }
    }
}
fn event_packet(id: u32, data: u32) -> Vec<u8> {
    packet(4, &u32s(&[0, id, data]))
}
fn filename_packet(id: u32, flight: Option<&str>) -> Vec<u8> {
    let mut b = u32s(&[0, id, 0]);
    let mut name = [0u8; 260];
    if let Some(f) = flight {
        let bytes = f.as_bytes();
        name[..bytes.len()].copy_from_slice(bytes);
    }
    b.extend(name);
    b.extend(0u32.to_le_bytes());
    packet(6, &b)
}
fn state_packet(id: u32, integer: u32, flight: Option<&str>) -> Vec<u8> {
    let mut b = u32s(&[id, integer, 0]);
    let mut name = [0u8; 260];
    if let Some(f) = flight {
        let bytes = f.as_bytes();
        name[..bytes.len()].copy_from_slice(bytes);
    }
    b.extend(name);
    packet(15, &b)
}
fn advance(s: &mut State, scenario: &Scenario, from: u64, to: u64, notification: usize) {
    if to < from {
        return;
    }
    if scenario.events.is_empty() {
        s.life.through = to;
        return;
    }
    let first = from / scenario.duration;
    let last = to / scenario.duration;
    if last - first > MAX_QUEUE as u64 {
        s.overflow = true;
        return;
    }
    for pass in first..=last {
        for ev in &scenario.events {
            let Some(at) = pass
                .checked_mul(scenario.duration)
                .and_then(|v| v.checked_add(ev.at))
            else {
                continue;
            };
            if at < from || at > to {
                continue;
            }
            apply_event(s, &ev.event, at, notification);
        }
    }
    s.life.through = to;
}
fn apply_event(s: &mut State, event: &Event, at: u64, notification: usize) {
    match event {
        Event::Pause(p) => {
            s.life.pause = p.flags();
            if at >= s.life.unavailable_until
                && let Some(&id) = s.subscriptions.get("Pause_EX1")
            {
                send(s, notification, event_packet(id, s.life.pause));
            }
        }
        Event::Flight(v) => {
            s.life.sim = *v;
            if at >= s.life.unavailable_until
                && let Some(&id) = s.subscriptions.get("Sim")
            {
                send(s, notification, event_packet(id, u32::from(*v)));
            }
        }
        Event::FlightLoaded(f) => {
            s.life.flight = f.clone();
            if at >= s.life.unavailable_until
                && let Some(&id) = s.subscriptions.get("FlightLoaded")
            {
                send(s, notification, filename_packet(id, f.as_deref()));
            }
        }
        Event::Disconnect(duration) => {
            s.life.unavailable_until = at.saturating_add(*duration);
            send(s, notification, packet(3, &[]));
        }
        Event::RawPacket {
            bytes,
            returned_length,
        } => {
            if at >= s.life.unavailable_until {
                send_raw(s, notification, bytes.clone(), *returned_length);
            }
        }
    }
}
fn emit(s: &mut State, scenario: &Scenario, elapsed: u64, notification: usize) {
    enum Due {
        Event(Event),
        Packet(Vec<u8>),
    }
    let mut due: Vec<(usize, Due)> = scenario
        .events
        .iter()
        .filter(|e| e.at == elapsed % scenario.duration)
        .map(|e| (e.order, Due::Event(e.event.clone())))
        .collect();
    for &(request, definition) in &s.requests {
        let Some(fields) = s.definitions.get(&definition) else {
            continue;
        };
        let values: Option<Vec<(f64, usize)>> = fields
            .iter()
            .map(|(n, u)| scenario.ordered_value(n, u, elapsed))
            .collect();
        if let Some(values) = values {
            let order = values.iter().map(|v| v.1).max().unwrap_or(0);
            let mut b = u32s(&[request, 0, definition, 0, 1, 1, values.len() as u32]);
            for (value, _) in values {
                b.extend(value.to_le_bytes());
            }
            due.push((order, Due::Packet(packet(8, &b))));
        }
    }
    due.sort_by_key(|v| v.0);
    for (_, item) in due {
        match item {
            Due::Event(event) => apply_event(s, &event, elapsed, notification),
            Due::Packet(packet) => {
                if elapsed >= s.life.unavailable_until {
                    send(s, notification, packet)
                }
            }
        }
    }
}
fn thread_main(
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    scenario: Scenario,
    origin: Instant,
    notification: usize,
) {
    let mut last_frame =
        origin.elapsed().as_micros().min(u64::MAX as u128) as u64 / scenario.frame_us;
    while !stop.load(Ordering::Acquire) {
        let now = origin.elapsed().as_micros().min(u64::MAX as u128) as u64;
        let frame = now / scenario.frame_us;
        {
            let mut s = state.lock().unwrap_or_else(|p| p.into_inner());
            let from = s.life.through.saturating_add(1);
            if frame > last_frame {
                let deadline = frame * scenario.frame_us;
                if deadline > from {
                    advance(&mut s, &scenario, from, deadline - 1, notification);
                }
                emit(&mut s, &scenario, deadline, notification);
                s.life.through = deadline;
                if now > deadline {
                    advance(&mut s, &scenario, deadline + 1, now, notification);
                }
                last_frame = frame;
            } else {
                advance(&mut s, &scenario, from, now, notification);
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}
fn load_scenario() -> Result<Scenario, String> {
    let mut args = std::env::args_os().skip(1);
    let mut path = None;
    while let Some(arg) = args.next() {
        if arg == "--" {
            break;
        }
        let wide: Vec<u16> = arg.encode_wide().collect();
        let prefix: Vec<u16> = "--scenario=".encode_utf16().collect();
        let value = if arg == "--scenario" {
            Some(args.next().ok_or("--scenario requires a path")?)
        } else {
            wide.strip_prefix(prefix.as_slice())
                .map(OsString::from_wide)
        };
        if let Some(value) = value
            && path.replace(PathBuf::from(value)).is_some()
        {
            return Err("--scenario specified more than once".into());
        }
    }
    let p = path.ok_or("--scenario is required")?;
    let file = std::fs::File::open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{}: {e}", p.display()))?;
    let text =
        String::from_utf8(bytes).map_err(|e| format!("{}: invalid UTF-8: {e}", p.display()))?;
    Scenario::parse(&text, &p.display().to_string())
}
/// # Safety
/// The output must be writable, and the notification handle must remain valid until Close returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_Open(
    out: *mut *mut c_void,
    _name: *const c_char,
    _window: *mut c_void,
    _event_id: u32,
    notification: *mut c_void,
    _config: u32,
) -> i32 {
    guard(|| {
        if out.is_null() {
            return FAIL;
        }
        unsafe { *out = ptr::null_mut() };
        let mut g = global().lock().unwrap_or_else(|p| p.into_inner());
        if g.active {
            return FAIL;
        }
        if g.scenario.is_none() {
            match load_scenario() {
                Ok(scenario) => {
                    g.fault_counts = vec![0; scenario.faults.len()];
                    g.scenario = Some(scenario);
                }
                Err(error) => {
                    let _ = writeln!(std::io::stderr().lock(), "Demo scenario: {error}");
                    return DEMO_CONFIGURATION_ERROR;
                }
            }
        }
        let scenario = g.scenario.clone().expect("scenario was loaded");
        if let Some(hr) = g.fault("SimConnect_Open", None, None) {
            return hr;
        }
        let now = g.origin.map_or(0, |origin| {
            origin.elapsed().as_micros().min(u64::MAX as u128) as u64
        });
        let life = g.lifecycle.clone().unwrap_or(Life {
            sim: scenario.initial.sim_running,
            pause: scenario.initial.pause.flags(),
            flight: scenario.initial.flight.clone(),
            through: 0,
            unavailable_until: 0,
        });
        let mut state = State {
            queue: VecDeque::new(),
            retained: vec![],
            definitions: HashMap::new(),
            requests: Vec::new(),
            subscriptions: HashMap::new(),
            last_id: 0,
            life,
            overflow: false,
        };
        let from = if g.lifecycle.is_some() {
            state.life.through.saturating_add(1)
        } else {
            0
        };
        advance(&mut state, &scenario, from, now, 0);
        let initial_packets = if g.lifecycle.is_none() {
            std::mem::take(&mut state.queue)
        } else {
            state.queue.clear();
            VecDeque::new()
        };
        if g.origin.is_some() && now < state.life.unavailable_until {
            g.lifecycle = Some(state.life);
            return FAIL;
        }
        let mut open = vec![0u8; 296];
        open[..8].copy_from_slice(b"Demo DLL");
        send(&mut state, notification as usize, packet(2, &open));
        for (bytes, len) in initial_packets {
            send_raw(&mut state, notification as usize, bytes, len);
        }
        let state = Arc::new(Mutex::new(state));
        let stop = Arc::new(AtomicBool::new(false));
        let origin = g.origin.unwrap_or_else(Instant::now);
        let st = state.clone();
        let flag = stop.clone();
        let handle = notification as usize;
        let worker = thread::spawn(move || thread_main(st, flag, scenario, origin, handle));
        let c = Box::new(Connection {
            state,
            stop,
            thread: Some(worker),
            notification: handle,
        });
        g.origin = Some(origin);
        g.active = true;
        unsafe { *out = Box::into_raw(c).cast() };
        0
    })
}
/// # Safety
/// The handle must be a live connection returned by Open and must not be used afterward.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_Close(h: *mut c_void) -> i32 {
    guard(|| {
        if h.is_null() {
            return FAIL;
        }
        let mut c = unsafe { Box::from_raw(h as *mut Connection) };
        c.stop.store(true, Ordering::Release);
        if c.thread.take().is_some_and(|t| t.join().is_err()) {
            return FAIL;
        }
        let life = c
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .life
            .clone();
        let mut g = global().lock().unwrap_or_else(|p| p.into_inner());
        g.lifecycle = Some(life);
        g.active = false;
        0
    })
}
/// # Safety
/// The handle must be live and both output pointers writable. Returned bytes remain valid until the next dispatch or Close.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_GetNextDispatch(
    h: *mut c_void,
    out: *mut *mut c_void,
    len: *mut u32,
) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        if out.is_null() || len.is_null() {
            return FAIL;
        }
        let mut s = c.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.overflow {
            return 0x8000ffff_u32 as i32;
        }
        let Some((p, reported_len)) = s.queue.pop_front() else {
            unsafe {
                *out = ptr::null_mut();
                *len = 0
            };
            return FAIL;
        };
        s.retained = p;
        unsafe {
            *out = s.retained.as_mut_ptr().cast();
            *len = reported_len
        };
        if !s.queue.is_empty() && c.notification != 0 {
            unsafe {
                SetEvent(c.notification as *mut c_void);
            }
        }
        0
    })
}
/// # Safety
/// The handle must be live and both strings must be readable, NUL-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_AddToDataDefinition(
    h: *mut c_void,
    id: u32,
    name: *const c_char,
    unit: *const c_char,
    datatype: i32,
    epsilon: f32,
    datum: u32,
) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        let (Some(n), Some(u)) = (unsafe { string(name) }, unsafe { string(unit) }) else {
            return FAIL;
        };
        if !scenario::CATALOG.contains(&(n.as_str(), u.as_str()))
            || datatype != 4
            || epsilon != 0.0
            || datum != u32::MAX
        {
            return FAIL;
        }
        if let Some(hr) = global().lock().unwrap_or_else(|p| p.into_inner()).fault(
            "SimConnect_AddToDataDefinition",
            Some(&n),
            Some(&u),
        ) {
            return hr;
        }
        let mut s = c.state.lock().unwrap_or_else(|p| p.into_inner());
        s.definitions.entry(id).or_default().push((n, u));
        s.last_id = s.last_id.wrapping_add(1);
        0
    })
}
/// # Safety
/// The handle must be a live connection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_RequestDataOnSimObject(
    h: *mut c_void,
    request: u32,
    definition: u32,
    object: u32,
    period: i32,
    flags: u32,
    origin: u32,
    interval: u32,
    limit: u32,
) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        if object != 0 || period != 3 || flags != 0 || origin != 0 || interval != 0 || limit != 0 {
            return FAIL;
        }
        let mut s = c.state.lock().unwrap_or_else(|p| p.into_inner());
        if !s.definitions.contains_key(&definition) {
            return FAIL;
        }
        if let Some(entry) = s.requests.iter_mut().find(|(id, _)| *id == request) {
            entry.1 = definition;
        } else {
            s.requests.push((request, definition));
        }
        s.last_id = s.last_id.wrapping_add(1);
        0
    })
}
/// # Safety
/// The handle must be live and the name must be a readable, NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_SubscribeToSystemEvent(
    h: *mut c_void,
    id: u32,
    name: *const c_char,
) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        let Some(n) = (unsafe { string(name) }) else {
            return FAIL;
        };
        if !["Sim", "Pause_EX1", "FlightLoaded"].contains(&n.as_str()) {
            return FAIL;
        }
        if let Some(hr) = global().lock().unwrap_or_else(|p| p.into_inner()).fault(
            "SimConnect_SubscribeToSystemEvent",
            None,
            None,
        ) {
            return hr;
        }
        let mut s = c.state.lock().unwrap_or_else(|p| p.into_inner());
        s.subscriptions.insert(n.clone(), id);
        s.last_id = s.last_id.wrapping_add(1);
        if n == "Pause_EX1" {
            let pause = s.life.pause;
            send(&mut s, c.notification, event_packet(id, pause));
        }
        0
    })
}
/// # Safety
/// The handle must be live and the name must be a readable, NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_RequestSystemState(
    h: *mut c_void,
    id: u32,
    name: *const c_char,
) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        let Some(n) = (unsafe { string(name) }) else {
            return FAIL;
        };
        let mut s = c.state.lock().unwrap_or_else(|p| p.into_inner());
        let p = match n.as_str() {
            "Sim" => state_packet(id, u32::from(s.life.sim), None),
            "FlightLoaded" => state_packet(id, 0, s.life.flight.as_deref()),
            _ => return FAIL,
        };
        s.last_id = s.last_id.wrapping_add(1);
        send(&mut s, c.notification, p);
        0
    })
}
/// # Safety
/// The handle must be live and the output pointer writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SimConnect_GetLastSentPacketID(h: *mut c_void, out: *mut u32) -> i32 {
    guard(|| {
        let Some(c) = (unsafe { conn(h) }) else {
            return FAIL;
        };
        if out.is_null() {
            return FAIL;
        }
        unsafe { *out = c.state.lock().unwrap_or_else(|p| p.into_inner()).last_id };
        0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fault_occurrences_count_only_matching_calls_and_do_not_rearm() {
        let text = concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":1000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":null}}\n",
            "{\"type\":\"api_fault\",\"operation\":\"SimConnect_AddToDataDefinition\",\"occurrence\":2,\"hresult\":-42,\"selector\":{\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\"}}\n"
        );
        let scenario = Scenario::parse(text, "test").unwrap();
        let mut global = Global::new();
        global.fault_counts = vec![0; scenario.faults.len()];
        global.scenario = Some(scenario);
        assert_eq!(
            global.fault(
                "SimConnect_AddToDataDefinition",
                Some("AIRSPEED TRUE"),
                Some("knots")
            ),
            None
        );
        assert_eq!(
            global.fault(
                "SimConnect_AddToDataDefinition",
                Some("AIRSPEED INDICATED"),
                Some("knots")
            ),
            None
        );
        assert_eq!(
            global.fault(
                "SimConnect_AddToDataDefinition",
                Some("AIRSPEED INDICATED"),
                Some("knots")
            ),
            Some(-42)
        );
        assert_eq!(
            global.fault(
                "SimConnect_AddToDataDefinition",
                Some("AIRSPEED INDICATED"),
                Some("knots")
            ),
            None
        );
    }
    #[test]
    fn events_persist_across_pass_without_implicit_reset() {
        let text = concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":1000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"one\"}}\n",
            "{\"type\":\"pause\",\"at_us\":500000,\"mode\":\"active\"}\n",
            "{\"type\":\"flight\",\"at_us\":600000,\"active\":false}\n"
        );
        let scenario = Scenario::parse(text, "test").unwrap();
        let mut s = State {
            queue: VecDeque::new(),
            retained: vec![],
            definitions: HashMap::new(),
            requests: Vec::new(),
            subscriptions: HashMap::new(),
            last_id: 0,
            life: Life {
                sim: true,
                pause: 0,
                flight: Some("one".into()),
                through: 0,
                unavailable_until: 0,
            },
            overflow: false,
        };
        s.subscriptions.insert("Pause_EX1".into(), 42);
        s.subscriptions.insert("Sim".into(), 71);
        advance(&mut s, &scenario, 0, 700_000, 0);
        assert_eq!(s.life.pause, 4);
        assert!(!s.life.sim);
        assert_eq!(s.queue.len(), 2);
        advance(&mut s, &scenario, 700_001, 1_400_000, 0);
        assert_eq!(s.life.pause, 4);
        assert!(!s.life.sim);
        assert_eq!(s.queue.len(), 2);
        advance(&mut s, &scenario, 1_400_001, 1_700_000, 0);
        assert_eq!(s.queue.len(), 4);
    }
}

#[cfg(test)]
mod ordering_tests {
    use super::*;
    #[test]
    fn independent_tracks_emit_equal_holds_and_gaps_on_controlled_frames() {
        let input = r#"{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":8000000,"initial":{"sim_running":true,"pause":"off","flight":"one"}}
{"type":"signal","id":"ias","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":true}
{"type":"hold","signal":"ias","value":100,"duration_ms":1500}
{"type":"gap","signal":"ias","duration_ms":1500}
{"type":"signal","id":"tas","simvar":"AIRSPEED TRUE","unit":"knots","repeat":true}
{"type":"hold","signal":"tas","value":10,"duration_ms":1000}
{"type":"hold","signal":"tas","value":20,"duration_ms":1000}"#;
        let scenario = Scenario::parse(input, "controlled").unwrap();
        let mut s = State {
            queue: VecDeque::new(),
            retained: vec![],
            definitions: HashMap::from([
                (1, vec![("AIRSPEED INDICATED".into(), "knots".into())]),
                (2, vec![("AIRSPEED TRUE".into(), "knots".into())]),
            ]),
            requests: vec![(11, 1), (12, 2)],
            subscriptions: HashMap::new(),
            last_id: 0,
            life: Life {
                sim: true,
                pause: 0,
                flight: None,
                through: 0,
                unavailable_until: 0,
            },
            overflow: false,
        };
        for frame in 0..160_u64 {
            emit(&mut s, &scenario, frame * 50_000, 0);
            let mut values = HashMap::new();
            while let Some((packet, _)) = s.queue.pop_front() {
                let id = u32::from_le_bytes(packet[12..16].try_into().unwrap());
                let value = f64::from_le_bytes(packet[40..48].try_into().unwrap());
                assert!(values.insert(id, value).is_none());
            }
            let ias_offset = frame % 60;
            assert_eq!(
                values.get(&11).copied(),
                (ias_offset < 30).then_some(100.0),
                "frame {frame}"
            );
            assert_eq!(
                values.get(&12).copied(),
                Some(if frame % 40 < 20 { 10.0 } else { 20.0 }),
                "frame {frame}"
            );
        }
        // The pass boundary restarts the three-second track at eight seconds.
        emit(&mut s, &scenario, 8_000_000, 0);
        assert_eq!(
            u32::from_le_bytes(s.queue[0].0[12..16].try_into().unwrap()),
            11
        );
    }
    #[test]
    fn grouped_packets_follow_last_contributing_record_and_gaps_suppress_them() {
        let header = r#"{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":1000000,"initial":{"sim_running":true,"pause":"off","flight":"one"}}"#;
        let pitch = r#"{"type":"signal","id":"p","simvar":"PLANE PITCH DEGREES","unit":"radians","repeat":true}
{"type":"hold","signal":"p","value":0.5,"duration_ms":1000}"#;
        let bank = r#"{"type":"signal","id":"b","simvar":"PLANE BANK DEGREES","unit":"radians","repeat":true}
{"type":"hold","signal":"b","value":1,"duration_ms":1000}"#;
        let gap = r#"{"type":"signal","id":"b","simvar":"PLANE BANK DEGREES","unit":"radians","repeat":true}
{"type":"gap","signal":"b","duration_ms":1000}"#;
        let event = r#"{"type":"pause","at_us":50000,"mode":"active"}"#;
        for (parts, expected) in [
            (vec![pitch, event, bank], vec![4, 8]),
            (vec![bank, pitch, event], vec![8, 4]),
            (vec![bank, event, pitch], vec![4, 8]),
            (vec![pitch, event, gap], vec![4]),
        ] {
            let scenario =
                Scenario::parse(&format!("{header}\n{}", parts.join("\n")), "test").unwrap();
            let mut s = State {
                queue: VecDeque::new(),
                retained: vec![],
                definitions: HashMap::from([(
                    7,
                    vec![
                        ("PLANE PITCH DEGREES".into(), "radians".into()),
                        ("PLANE BANK DEGREES".into(), "radians".into()),
                    ],
                )]),
                requests: vec![(9, 7)],
                subscriptions: HashMap::from([("Pause_EX1".into(), 10)]),
                last_id: 0,
                life: Life {
                    sim: true,
                    pause: 0,
                    flight: None,
                    through: 0,
                    unavailable_until: 0,
                },
                overflow: false,
            };
            emit(&mut s, &scenario, 50_000, 0);
            let ids: Vec<u32> = s
                .queue
                .iter()
                .map(|p| u32::from_le_bytes(p.0[8..12].try_into().unwrap()))
                .collect();
            assert_eq!(ids, expected);
            assert_eq!(s.life.pause, 4);
        }
    }
}
