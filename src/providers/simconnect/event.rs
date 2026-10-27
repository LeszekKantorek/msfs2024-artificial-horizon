//! Windows event ownership for notification, interruptible waits, and cancellation.
use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{CreateEventW, SetEvent, WaitForMultipleObjects},
};

pub(super) struct Event(HANDLE);

// Windows event operations are thread-safe. Arc ownership keeps the handle open
// until the worker and cancellation owner have both stopped using it.
unsafe impl Send for Event {}
unsafe impl Sync for Event {}

pub(super) enum Wake {
    Stop,
    Notification,
    Timeout,
}

impl Event {
    pub(super) fn new(manual_reset: bool) -> io::Result<Self> {
        // SAFETY: no name/security pointers; ownership of the returned handle is unique.
        let handle = unsafe { CreateEventW(ptr::null(), i32::from(manual_reset), 0, ptr::null()) };
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }

    pub(super) fn handle(&self) -> HANDLE {
        self.0
    }

    pub(super) fn signal(&self) -> io::Result<()> {
        // SAFETY: self owns a live event handle.
        if unsafe { SetEvent(self.0) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn wait(&self, notification: Option<&Event>, millis: u32) -> io::Result<Wake> {
        let handles = [
            self.0,
            notification.map_or(ptr::null_mut(), |event| event.0),
        ];
        let count = if notification.is_some() { 2 } else { 1 };
        // SAFETY: count selects only live event handles, borrowed for the whole wait.
        match unsafe { WaitForMultipleObjects(count, handles.as_ptr(), 0, millis) } {
            WAIT_OBJECT_0 => Ok(Wake::Stop),
            value if notification.is_some() && value == WAIT_OBJECT_0 + 1 => Ok(Wake::Notification),
            WAIT_TIMEOUT => Ok(Wake::Timeout),
            WAIT_FAILED => Err(io::Error::last_os_error()),
            value => Err(io::Error::other(format!(
                "Unexpected event wait result: {value}"
            ))),
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: unique handle, no wait can outlive its Event/Arc borrow.
        unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_interrupts_wait_and_has_priority_over_notification() {
        let stop = Event::new(true).unwrap();
        let data = Event::new(false).unwrap();
        assert!(matches!(stop.wait(Some(&data), 0).unwrap(), Wake::Timeout));
        data.signal().unwrap();
        assert!(matches!(
            stop.wait(Some(&data), 0).unwrap(),
            Wake::Notification
        ));
        data.signal().unwrap();
        stop.signal().unwrap();
        assert!(matches!(stop.wait(Some(&data), 1000).unwrap(), Wake::Stop));
        assert!(matches!(stop.wait(None, 0).unwrap(), Wake::Stop));
    }
}
