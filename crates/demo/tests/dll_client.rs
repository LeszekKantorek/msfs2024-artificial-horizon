use std::os::windows::ffi::OsStrExt;
use std::{
    ffi::{c_char, c_void},
    ptr, thread,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FreeLibrary, WAIT_OBJECT_0},
    System::{
        LibraryLoader::{GetProcAddress, LoadLibraryW},
        Threading::{CreateEventW, WaitForSingleObject},
    },
};
type Configure = unsafe extern "C" fn(*const u16, *mut u8, u32) -> i32;
type Open = unsafe extern "C" fn(
    *mut *mut c_void,
    *const c_char,
    *mut c_void,
    u32,
    *mut c_void,
    u32,
) -> i32;
type Close = unsafe extern "C" fn(*mut c_void) -> i32;
type Next = unsafe extern "C" fn(*mut c_void, *mut *mut c_void, *mut u32) -> i32;
type Add =
    unsafe extern "C" fn(*mut c_void, u32, *const c_char, *const c_char, i32, f32, u32) -> i32;
type Subscribe = unsafe extern "C" fn(*mut c_void, u32, *const c_char) -> i32;
type Query = unsafe extern "C" fn(*mut c_void, u32, *const c_char) -> i32;
type LastSent = unsafe extern "C" fn(*mut c_void, *mut u32) -> i32;
type Request = unsafe extern "C" fn(*mut c_void, u32, u32, u32, i32, u32, u32, u32, u32) -> i32;
#[test]
fn actual_dll_notifies_and_retains_packets() {
    unsafe {
        let path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("demo.dll");
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let dll = LoadLibraryW(wide.as_ptr());
        assert!(!dll.is_null(), "build the debug DLL first");
        macro_rules! sym {
            ($n:literal,$ty:ty) => {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(
                    GetProcAddress(dll, concat!($n, "\0").as_ptr()).unwrap(),
                )
            };
        }
        let setup: Configure = sym!("EfiDemo_ConfigureV1", Configure);
        let open: Open = sym!("SimConnect_Open", Open);
        let close: Close = sym!("SimConnect_Close", Close);
        let next: Next = sym!("SimConnect_GetNextDispatch", Next);
        let add: Add = sym!("SimConnect_AddToDataDefinition", Add);
        let request: Request = sym!("SimConnect_RequestDataOnSimObject", Request);
        let subscribe: Subscribe = sym!("SimConnect_SubscribeToSystemEvent", Subscribe);
        let query: Query = sym!("SimConnect_RequestSystemState", Query);
        let last_sent: LastSent = sym!("SimConnect_GetLastSentPacketID", LastSent);
        let mut error = [0u8; 512];
        assert_eq!(
            setup(ptr::null(), error.as_mut_ptr(), error.len() as u32),
            0
        );
        let event = CreateEventW(ptr::null(), 0, 0, ptr::null());
        assert!(!event.is_null());
        let mut h = ptr::null_mut();
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(
            add(
                h,
                77,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        let mut definition_packet = 0;
        assert_eq!(last_sent(h, &mut definition_packet), 0);
        assert_ne!(definition_packet, 0);
        assert!(
            add(
                h,
                78,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knot".as_ptr(),
                4,
                0.0,
                u32::MAX
            ) < 0
        );
        let mut after_rejection = 0;
        assert_eq!(last_sent(h, &mut after_rejection), 0);
        assert_eq!(after_rejection, definition_packet);
        assert_eq!(request(h, 991, 77, 0, 3, 0, 0, 0, 0), 0);
        let mut request_packet = 0;
        assert_eq!(last_sent(h, &mut request_packet), 0);
        assert_ne!(request_packet, definition_packet);
        let mut unchanged = 0;
        assert_eq!(last_sent(h, &mut unchanged), 0);
        assert_eq!(unchanged, request_packet);
        let mut p = ptr::null_mut();
        let mut n = 0;
        assert_eq!(WaitForSingleObject(event, 1000), WAIT_OBJECT_0);
        assert_eq!(next(h, &mut p, &mut n), 0);
        assert_eq!(n, 308);
        let saved = std::slice::from_raw_parts(p as *const u8, n as usize).to_vec();
        thread::sleep(Duration::from_millis(160));
        assert_eq!(
            std::slice::from_raw_parts(p as *const u8, n as usize),
            saved
        );
        let mut samples = 0;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8 {
                assert_eq!(u32::from_le_bytes(b[12..16].try_into().unwrap()), 991);
                assert_eq!(u32::from_le_bytes(b[20..24].try_into().unwrap()), 77);
                assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 0.0);
                samples += 1;
            }
        }
        assert!(samples >= 2, "equal frame observations: {samples}");
        // Consume any coalesced signal, then wait for an independently produced frame.
        while WaitForSingleObject(event, 0) == WAIT_OBJECT_0 {}
        assert_eq!(WaitForSingleObject(event, 1000), WAIT_OBJECT_0);
        assert_eq!(next(h, &mut p, &mut n), 0);
        assert_eq!(
            u32::from_le_bytes(
                std::slice::from_raw_parts(p as *const u8, n as usize)[8..12]
                    .try_into()
                    .unwrap()
            ),
            8
        );
        assert_eq!(close(h), 0);
        let scenario =
            std::env::temp_dir().join(format!("demo-dll-test-{}.jsonl", std::process::id()));
        std::fs::write(&scenario, concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":5000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
            "{\"type\":\"signal\",\"id\":\"p\",\"simvar\":\"PLANE PITCH DEGREES\",\"unit\":\"radians\",\"repeat\":true}\n",
            "{\"type\":\"hold\",\"signal\":\"p\",\"value\":0.5,\"duration_ms\":5000}\n",
            "{\"type\":\"signal\",\"id\":\"b\",\"simvar\":\"PLANE BANK DEGREES\",\"unit\":\"radians\",\"repeat\":true}\n",
            "{\"type\":\"hold\",\"signal\":\"b\",\"value\":1.0,\"duration_ms\":5000}\n"
        )).unwrap();
        let scenario_wide: Vec<u16> = scenario.as_os_str().encode_wide().chain([0]).collect();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        let catalog = [
            ("AIRSPEED INDICATED", "knots"),
            ("AIRSPEED TRUE", "knots"),
            ("GROUND VELOCITY", "knots"),
            ("INDICATED ALTITUDE:1", "feet"),
            ("KOHLSMAN SETTING HG:1", "inHg"),
            ("VERTICAL SPEED", "feet per minute"),
            ("PLANE HEADING DEGREES MAGNETIC", "degrees"),
            ("TURN COORDINATOR BALL", "position 128"),
            ("TURN INDICATOR RATE", "degrees per second"),
            ("DESIGN SPEED VS0", "knots"),
            ("DESIGN SPEED VS1", "knots"),
            ("DESIGN SPEED MIN ROTATION", "knots"),
            ("AIRSPEED BARBER POLE", "knots"),
            ("AUTOPILOT AVAILABLE", "bool"),
            ("AUTOPILOT HEADING LOCK DIR:0", "degrees"),
            ("AUTOPILOT ALTITUDE LOCK VAR:0", "feet"),
            ("AUTOPILOT ALT RADIO MODE", "bool"),
        ];
        for (i, (name, unit)) in catalog.iter().enumerate() {
            let name = std::ffi::CString::new(*name).unwrap();
            let unit = std::ffi::CString::new(*unit).unwrap();
            let id = 200 + i as u32;
            assert_eq!(
                add(h, id, name.as_ptr(), unit.as_ptr(), 4, 0.0, u32::MAX),
                0
            );
            assert_eq!(request(h, id, id, 0, 3, 0, 0, 0, 0), 0);
        }
        for (name, unit) in [
            (c"PLANE PITCH DEGREES", c"radians"),
            (c"PLANE PITCH DEGREES", c"degrees"),
            (c"PLANE BANK DEGREES", c"radians"),
            (c"PLANE BANK DEGREES", c"degrees"),
            (c"PLANE PITCH DEGREES", c"radians"),
            (c"PLANE BANK DEGREES", c"radians"),
        ] {
            assert_eq!(
                add(h, 53, name.as_ptr(), unit.as_ptr(), 4, 0.0, u32::MAX),
                0
            );
        }
        assert_eq!(request(h, 32, 53, 0, 3, 0, 0, 0, 0), 0);
        assert!(request(h, 33, 53, 0, 3, 1, 0, 0, 0) < 0);
        thread::sleep(Duration::from_millis(120));
        let mut found = false;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) != 8 {
                continue;
            }
            assert_eq!(n, 88);
            assert_eq!(u32::from_le_bytes(b[12..16].try_into().unwrap()), 32);
            let values: Vec<f64> = b[40..88]
                .as_chunks::<8>()
                .0
                .iter()
                .map(|v| f64::from_le_bytes(*v))
                .collect();
            let expected = [
                0.5,
                0.5_f64.to_degrees(),
                1.0,
                1.0_f64.to_degrees(),
                0.5,
                1.0,
            ];
            for (actual, expected) in values.iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-10);
            }
            found = true;
        }
        assert!(found);
        assert_eq!(close(h), 0);
        let mut text = String::from(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":5000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
        );
        for (i, (name, unit)) in catalog.iter().enumerate() {
            text.push_str(&format!("{{\"type\":\"signal\",\"id\":\"f{i}\",\"simvar\":\"{name}\",\"unit\":\"{unit}\",\"repeat\":true}}\n{{\"type\":\"hold\",\"signal\":\"f{i}\",\"value\":{},\"duration_ms\":5000}}\n", i + 1));
        }
        std::fs::write(&scenario, text).unwrap();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        for (i, (name, unit)) in catalog.iter().enumerate() {
            let name = std::ffi::CString::new(*name).unwrap();
            let unit = std::ffi::CString::new(*unit).unwrap();
            let id = 300 + i as u32;
            assert_eq!(
                add(h, id, name.as_ptr(), unit.as_ptr(), 4, 0.0, u32::MAX),
                0
            );
            assert_eq!(request(h, id, id, 0, 3, 0, 0, 0, 0), 0);
        }
        thread::sleep(Duration::from_millis(130));
        let mut seen = [false; 17];
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8 {
                let index = (u32::from_le_bytes(b[12..16].try_into().unwrap()) - 300) as usize;
                assert_eq!(
                    f64::from_le_bytes(b[40..48].try_into().unwrap()),
                    (index + 1) as f64
                );
                seen[index] = true;
            }
        }
        assert!(seen.into_iter().all(|v| v));
        assert_eq!(close(h), 0);
        std::fs::write(&scenario, concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":3000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"old\"}}\n",
            "{\"type\":\"pause\",\"at_us\":200000,\"mode\":\"normal\"}\n",
            "{\"type\":\"pause\",\"at_us\":400000,\"mode\":\"active\"}\n",
            "{\"type\":\"flight\",\"at_us\":600000,\"active\":false}\n",
            "{\"type\":\"flight_loaded\",\"at_us\":800000,\"flight\":\"new\"}\n",
            "{\"type\":\"pause\",\"at_us\":1200000,\"mode\":\"off\"}\n",
            "{\"type\":\"flight\",\"at_us\":1400000,\"active\":true}\n",
            "{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":false}\n",
            "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":70,\"duration_ms\":3000}\n"
        )).unwrap();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(subscribe(h, 777, c"Pause_EX1".as_ptr()), 0);
        assert_eq!(subscribe(h, 778, c"Sim".as_ptr()), 0);
        assert_eq!(subscribe(h, 779, c"FlightLoaded".as_ptr()), 0);
        assert_eq!(
            add(
                h,
                40,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        assert_eq!(request(h, 41, 40, 0, 3, 0, 0, 0, 0), 0);
        thread::sleep(Duration::from_millis(1000));
        let mut pause_flags = Vec::new();
        let mut flight_event = false;
        let mut current_pause = 0;
        let mut paused_samples = [false; 2];
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            let id = u32::from_le_bytes(b[8..12].try_into().unwrap());
            if id == 4 && u32::from_le_bytes(b[16..20].try_into().unwrap()) == 777 {
                current_pause = u32::from_le_bytes(b[20..24].try_into().unwrap());
                pause_flags.push(current_pause);
            }
            if id == 6 {
                flight_event = true;
            }
            if id == 8 && u32::from_le_bytes(b[12..16].try_into().unwrap()) == 41 {
                assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 70.0);
                if current_pause == 1 {
                    paused_samples[0] = true;
                }
                if current_pause == 4 {
                    paused_samples[1] = true;
                }
            }
        }
        assert_eq!(pause_flags, vec![0, 1, 4]);
        assert!(flight_event);
        assert_eq!(paused_samples, [true, true]);
        assert_eq!(close(h), 0);
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(subscribe(h, 777, c"Pause_EX1".as_ptr()), 0);
        assert_eq!(query(h, 801, c"Sim".as_ptr()), 0);
        assert_eq!(query(h, 802, c"FlightLoaded".as_ptr()), 0);
        let mut pause = None;
        let mut sim = None;
        let mut flight = None;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            let id = u32::from_le_bytes(b[8..12].try_into().unwrap());
            if id == 4 {
                pause = Some(u32::from_le_bytes(b[20..24].try_into().unwrap()));
            }
            if id == 15 {
                let request = u32::from_le_bytes(b[12..16].try_into().unwrap());
                if request == 801 {
                    sim = Some(u32::from_le_bytes(b[16..20].try_into().unwrap()));
                }
                if request == 802 {
                    flight = Some(b[24..27].to_vec());
                }
            }
        }
        assert_eq!(pause, Some(4));
        assert_eq!(sim, Some(0));
        assert_eq!(flight, Some(b"new".to_vec()));
        assert_eq!(subscribe(h, 778, c"Sim".as_ptr()), 0);
        thread::sleep(Duration::from_millis(650));
        assert_eq!(query(h, 801, c"Sim".as_ptr()), 0);
        assert_eq!(query(h, 802, c"FlightLoaded".as_ptr()), 0);
        let mut resumed_pause = false;
        let mut resumed_sim = false;
        let mut queried_sim = false;
        let mut queried_flight = false;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            let kind = u32::from_le_bytes(b[8..12].try_into().unwrap());
            if kind == 4 {
                let id = u32::from_le_bytes(b[16..20].try_into().unwrap());
                let value = u32::from_le_bytes(b[20..24].try_into().unwrap());
                if id == 777 && value == 0 {
                    resumed_pause = true;
                }
                if id == 778 && value == 1 {
                    resumed_sim = true;
                }
            }
            if kind == 15 {
                let id = u32::from_le_bytes(b[12..16].try_into().unwrap());
                if id == 801 {
                    queried_sim = u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1;
                }
                if id == 802 {
                    queried_flight = &b[24..27] == b"new";
                }
            }
        }
        assert!(resumed_pause && resumed_sim && queried_sim && queried_flight);
        assert_eq!(close(h), 0);
        std::fs::write(&scenario, concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":2000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
            "{\"type\":\"disconnect\",\"at_us\":50000,\"unavailable_ms\":300}\n",
            "{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":false}\n",
            "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":123,\"duration_ms\":2000}\n"
        )).unwrap();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        thread::sleep(Duration::from_millis(120));
        let mut quit = false;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 3 {
                quit = true;
            }
        }
        assert!(quit);
        assert_eq!(close(h), 0);
        assert!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ) < 0
        );
        thread::sleep(Duration::from_millis(300));
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(subscribe(h, 777, c"Pause_EX1".as_ptr()), 0);
        assert_eq!(query(h, 801, c"Sim".as_ptr()), 0);
        assert_eq!(query(h, 802, c"FlightLoaded".as_ptr()), 0);
        assert_eq!(
            add(
                h,
                99,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        assert_eq!(request(h, 999, 99, 0, 3, 0, 0, 0, 0), 0);
        thread::sleep(Duration::from_millis(120));
        let mut saw_sample = false;
        let mut saw_sim = false;
        let mut saw_flight = false;
        let mut saw_pause = false;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            let kind = u32::from_le_bytes(b[8..12].try_into().unwrap());
            assert_ne!(kind, 3, "the same disconnect must not repeat on reconnect");
            if kind == 8 && u32::from_le_bytes(b[12..16].try_into().unwrap()) == 999 {
                assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 123.0);
                saw_sample = true;
            }
            if kind == 4 {
                saw_pause = u32::from_le_bytes(b[20..24].try_into().unwrap()) == 0;
            }
            if kind == 15 {
                let id = u32::from_le_bytes(b[12..16].try_into().unwrap());
                if id == 801 {
                    saw_sim = u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1;
                }
                if id == 802 {
                    saw_flight = &b[24..28] == b"test";
                }
            }
        }
        assert!(saw_sample && saw_sim && saw_flight && saw_pause);
        assert_eq!(close(h), 0);
        std::fs::write(&scenario, concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":10000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
            "{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":false}\n",
            "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":100,\"duration_ms\":2500}\n",
            "{\"type\":\"gap\",\"signal\":\"ias\",\"duration_ms\":1800}\n",
            "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":200,\"duration_ms\":5700}\n",
            "{\"type\":\"signal\",\"id\":\"tas\",\"simvar\":\"AIRSPEED TRUE\",\"unit\":\"knots\",\"repeat\":false}\n",
            "{\"type\":\"hold\",\"signal\":\"tas\",\"value\":50,\"duration_ms\":10000}\n"
        )).unwrap();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        let hold_open_started = std::time::Instant::now();
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        for (id, name) in [(901, c"AIRSPEED INDICATED"), (902, c"AIRSPEED TRUE")] {
            assert_eq!(
                add(h, id, name.as_ptr(), c"knots".as_ptr(), 4, 0.0, u32::MAX),
                0
            );
            assert_eq!(request(h, id, id, 0, 3, 0, 0, 0, 0), 0);
        }
        thread::sleep(Duration::from_millis(300));
        let mut hold_samples = 0;
        let mut first_observed = None;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8
                && u32::from_le_bytes(b[12..16].try_into().unwrap()) == 901
            {
                assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 100.0);
                hold_samples += 1;
                first_observed.get_or_insert_with(std::time::Instant::now);
            }
        }
        assert!(
            hold_samples >= 3,
            "only {hold_samples} equal observations in the initial hold"
        );
        let first_observed = first_observed.unwrap();
        thread::sleep(Duration::from_millis(1050));
        assert!(first_observed.elapsed() > Duration::from_millis(1000));
        // Drain all observations already queued before looking for a new one.
        while next(h, &mut p, &mut n) == 0 {}
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        let mut late_equal = false;
        while std::time::Instant::now() < deadline && !late_equal {
            while next(h, &mut p, &mut n) == 0 {
                let b = std::slice::from_raw_parts(p as *const u8, n as usize);
                if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8
                    && u32::from_le_bytes(b[12..16].try_into().unwrap()) == 901
                {
                    assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 100.0);
                    late_equal = true;
                }
            }
            if !late_equal {
                thread::sleep(Duration::from_millis(5));
            }
        }
        assert!(
            late_equal,
            "a new equal observation must arrive after one second"
        );
        thread::sleep(Duration::from_millis(2750).saturating_sub(hold_open_started.elapsed()));
        while next(h, &mut p, &mut n) == 0 {}
        let gap_started = std::time::Instant::now();
        thread::sleep(Duration::from_millis(1200));
        assert!(gap_started.elapsed() >= Duration::from_millis(1000));
        let mut other_samples = 0;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8 {
                let id = u32::from_le_bytes(b[12..16].try_into().unwrap());
                assert_ne!(id, 901, "IAS must stay silent during the gap");
                if id == 902 {
                    other_samples += 1;
                }
            }
        }
        assert!(
            other_samples >= 10,
            "independent TAS observations continue through the gap"
        );
        thread::sleep(Duration::from_millis(600));
        let mut recovered = false;
        while next(h, &mut p, &mut n) == 0 {
            let b = std::slice::from_raw_parts(p as *const u8, n as usize);
            if u32::from_le_bytes(b[8..12].try_into().unwrap()) == 8
                && u32::from_le_bytes(b[12..16].try_into().unwrap()) == 901
            {
                assert_eq!(f64::from_le_bytes(b[40..48].try_into().unwrap()), 200.0);
                recovered = true;
            }
        }
        assert!(recovered);
        assert_eq!(close(h), 0);
        std::fs::write(&scenario, concat!(
            "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":5000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
            "{\"type\":\"api_fault\",\"operation\":\"SimConnect_Open\",\"occurrence\":1,\"hresult\":-2147467259}\n",
            "{\"type\":\"api_fault\",\"operation\":\"SimConnect_AddToDataDefinition\",\"occurrence\":1,\"hresult\":-2147467259,\"selector\":{\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\"}}\n",
            "{\"type\":\"api_fault\",\"operation\":\"SimConnect_SubscribeToSystemEvent\",\"occurrence\":1,\"hresult\":-2147467259}\n",
            "{\"type\":\"raw_packet\",\"at_us\":50000,\"bytes_hex\":\"01020304\",\"returned_length\":2}\n"
        )).unwrap();
        assert_eq!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ),
            0
        );
        assert!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ) < 0
        );
        assert!(h.is_null());
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(
            add(
                h,
                71,
                c"AIRSPEED TRUE".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        assert!(
            add(
                h,
                72,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ) < 0
        );
        assert_eq!(
            add(
                h,
                72,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        assert!(subscribe(h, 81, c"Sim".as_ptr()) < 0);
        assert_eq!(subscribe(h, 81, c"Sim".as_ptr()), 0);
        thread::sleep(Duration::from_millis(100));
        let mut saw_raw = false;
        while next(h, &mut p, &mut n) == 0 {
            if n == 2 {
                assert!(!p.is_null());
                assert_eq!(std::slice::from_raw_parts(p as *const u8, 2), &[1, 2]);
                saw_raw = true;
            }
        }
        assert!(saw_raw);
        assert_eq!(close(h), 0);
        assert_eq!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ),
            0
        );
        assert_eq!(
            add(
                h,
                72,
                c"AIRSPEED INDICATED".as_ptr(),
                c"knots".as_ptr(),
                4,
                0.0,
                u32::MAX
            ),
            0
        );
        assert_eq!(subscribe(h, 81, c"Sim".as_ptr()), 0);
        assert_eq!(close(h), 0);
        std::fs::write(&scenario, "not json").unwrap();
        assert!(
            setup(
                scenario_wide.as_ptr(),
                error.as_mut_ptr(),
                error.len() as u32
            ) < 0
        );
        let explanation = std::ffi::CStr::from_ptr(error.as_ptr().cast()).to_string_lossy();
        assert!(explanation.contains("line 1"));
        assert!(
            open(
                &mut h,
                c"test".as_ptr(),
                ptr::null_mut(),
                0,
                event,
                u32::MAX
            ) < 0
        );
        std::fs::remove_file(scenario).unwrap();
        CloseHandle(event);
        FreeLibrary(dll);
    }
}
