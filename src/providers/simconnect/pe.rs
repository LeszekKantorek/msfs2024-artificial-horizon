//! Bounded, non-executing inspection of the selected x64 DLL's named exports.
use std::{fs::File, io::Read, path::Path};

const MAX_IMAGE: u64 = 64 * 1024 * 1024;
const REQUIRED: [&[u8]; 8] = [
    b"SimConnect_Open",
    b"SimConnect_Close",
    b"SimConnect_GetNextDispatch",
    b"SimConnect_AddToDataDefinition",
    b"SimConnect_RequestDataOnSimObject",
    b"SimConnect_SubscribeToSystemEvent",
    b"SimConnect_RequestSystemState",
    b"SimConnect_GetLastSentPacketID",
];

fn word(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}
fn dword(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn inspect(bytes: &[u8]) -> Result<(), String> {
    let invalid = || "LoadLibrary preflight: invalid or unsupported x64 PE DLL".to_string();
    let bad = || invalid();
    if bytes.get(..2) != Some(b"MZ") {
        return Err(invalid());
    }
    let pe = dword(bytes, 0x3c).ok_or_else(bad)? as usize;
    if bytes.get(pe..pe.checked_add(4).ok_or_else(bad)?) != Some(b"PE\0\0")
        || word(bytes, pe + 4) != Some(0x8664)
        || word(bytes, pe + 22).ok_or_else(bad)? & 0x2000 == 0
    {
        return Err(invalid());
    }
    let count = word(bytes, pe + 6).ok_or_else(bad)? as usize;
    if count == 0 || count > 96 {
        return Err(invalid());
    }
    let optional = pe.checked_add(24).ok_or_else(bad)?;
    let optional_len = word(bytes, pe + 20).ok_or_else(bad)? as usize;
    if optional_len < 120
        || word(bytes, optional) != Some(0x20b)
        || dword(bytes, optional + 108).unwrap_or(0) < 1
    {
        return Err(invalid());
    }
    let sections = optional.checked_add(optional_len).ok_or_else(bad)?;
    bytes
        .get(sections..sections.checked_add(count * 40).ok_or_else(bad)?)
        .ok_or_else(bad)?;
    let header_size = dword(bytes, optional + 60).ok_or_else(bad)? as usize;
    let rva = |address: u32, size: usize| -> Option<usize> {
        let addr = address as usize;
        if addr < header_size
            && addr.checked_add(size)? <= header_size
            && addr.checked_add(size)? <= bytes.len()
        {
            return Some(addr);
        }
        for i in 0..count {
            let section = sections + i * 40;
            let start = dword(bytes, section + 12)? as usize;
            let raw_size = dword(bytes, section + 16)? as usize;
            let raw = dword(bytes, section + 20)? as usize;
            if let Some(delta) = addr.checked_sub(start)
                && delta.checked_add(size)? <= raw_size
            {
                let offset = raw.checked_add(delta)?;
                if offset.checked_add(size)? <= bytes.len() {
                    return Some(offset);
                }
            }
        }
        None
    };
    let export_rva = dword(bytes, optional + 112).ok_or_else(bad)?;
    let directory = rva(export_rva, 40).ok_or_else(bad)?;
    let function_count = dword(bytes, directory + 20).ok_or_else(bad)? as usize;
    let name_count = dword(bytes, directory + 24).ok_or_else(bad)? as usize;
    if name_count > 1_000_000 || function_count > 1_000_000 {
        return Err(invalid());
    }
    let functions = rva(
        dword(bytes, directory + 28).ok_or_else(bad)?,
        function_count.checked_mul(4).ok_or_else(bad)?,
    )
    .ok_or_else(bad)?;
    let names = rva(
        dword(bytes, directory + 32).ok_or_else(bad)?,
        name_count.checked_mul(4).ok_or_else(bad)?,
    )
    .ok_or_else(bad)?;
    let ordinals = rva(
        dword(bytes, directory + 36).ok_or_else(bad)?,
        name_count.checked_mul(2).ok_or_else(bad)?,
    )
    .ok_or_else(bad)?;
    let mut found = [false; 8];
    for i in 0..name_count {
        let name_rva = dword(bytes, names + i * 4).ok_or_else(bad)?;
        rva(name_rva, 1).ok_or_else(bad)?;
        // Check the complete name within one mapped range.
        for (j, required) in REQUIRED.iter().enumerate() {
            if let Some(start) = rva(name_rva, required.len() + 1)
                && bytes.get(start..start + required.len()) == Some(*required)
                && bytes.get(start + required.len()) == Some(&0)
            {
                let ordinal = word(bytes, ordinals + i * 2).ok_or_else(bad)? as usize;
                if ordinal >= function_count
                    || dword(bytes, functions + ordinal * 4).unwrap_or(0) == 0
                {
                    return Err(invalid());
                }
                found[j] = true;
            }
        }
    }
    for (i, present) in found.iter().enumerate() {
        if !present {
            return Err(format!(
                "missing export {}",
                String::from_utf8_lossy(REQUIRED[i])
            ));
        }
    }
    Ok(())
}

pub(super) fn validate(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("LoadLibrary preflight: {e}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_IMAGE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("LoadLibrary preflight: {e}"))?;
    if bytes.len() as u64 > MAX_IMAGE {
        return Err("LoadLibrary preflight: DLL exceeds 64 MiB inspection limit".into());
    }
    inspect(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_and_non_pe_inputs() {
        for input in [&b""[..], &b"MZ"[..], &b"not a library"[..]] {
            assert!(inspect(input).is_err());
        }
    }
    #[test]
    fn examines_a_real_system_image_without_loading_it() {
        let path = Path::new(&std::env::var("SystemRoot").unwrap()).join("System32/kernel32.dll");
        assert!(
            validate(&path)
                .unwrap_err()
                .contains("missing export SimConnect_Open")
        );
    }
}
