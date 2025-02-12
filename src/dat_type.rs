/// Enumeration of known FFXIV DAT file types.
/// The value of each element represents the first 2 header bytes as a little-endian i32.
/// These bytes are known static values that differentiate file types.
///
/// File types may be referenced using a human readable descriptor -- `DATType::GoldSaucer` --
/// or the filename used by FFXIV -- `DATType::GS`. These methods are interchangable and considered
/// equivalent. `DATType::GoldSaucer == DATType::GS`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DATType {
    /// GEARSET.DAT
    Gearset = 0x0005,
    /// GS.DAT
    GoldSaucer = 0x000A,
    /// HOTBAR.DAT
    Hotbar = 0x0002,
    /// ITEMFDR.DAT
    ItemFinder = 0x0008,
    /// ITEMODR.DAT
    ItemOrder = 0x0007,
    /// KEYBIND.DAT
    Keybind = 0x0003,
    /// LOGFLTR.DAT
    LogFilter = 0x0004,
    /// MACRO.DAT (Character) & MACROSYS.DAT (Global)
    Macro = 0x0001,
    /// ACQ.DAT (Acquaintances?)
    RecentTells = 0x0006,
    /// UISAVE.DAT
    UISave = 0x0009,
    Unknown = 0,
}

/// Provides alises matching exact file names rather than human-readable descriptors.
impl DATType {
    pub const ACQ: DATType = DATType::RecentTells;
    pub const GEARSET: DATType = DATType::Gearset;
    pub const GS: DATType = DATType::GoldSaucer;
    pub const ITEMFDR: DATType = DATType::ItemFinder;
    pub const ITEMODR: DATType = DATType::ItemOrder;
    pub const KEYBIND: DATType = DATType::Keybind;
    pub const LOGFLTR: DATType = DATType::LogFilter;
    pub const MACRO: DATType = DATType::Macro;
    pub const MACROSYS: DATType = DATType::Macro;
    pub const UISAVE: DATType = DATType::UISave;
}

impl From<u32> for DATType {
    fn from(x: u32) -> DATType {
        match x {
            x if x == DATType::Gearset as u32 => DATType::Gearset,
            x if x == DATType::GoldSaucer as u32 => DATType::GoldSaucer,
            x if x == DATType::Hotbar as u32 => DATType::Hotbar,
            x if x == DATType::ItemFinder as u32 => DATType::ItemFinder,
            x if x == DATType::ItemOrder as u32 => DATType::ItemOrder,
            x if x == DATType::Keybind as u32 => DATType::Keybind,
            x if x == DATType::LogFilter as u32 => DATType::LogFilter,
            x if x == DATType::Macro as u32 => DATType::Macro,
            x if x == DATType::RecentTells as u32 => DATType::RecentTells,
            x if x == DATType::UISave as u32 => DATType::UISave,
            _ => DATType::Unknown,
        }
    }
}

/// Gets the XOR mask used for the contents of a binary DAT file.
/// The mask is applied to only the file data content, not the header, footer, or padding null bytes.
/// Returns `None` if the file is of unknown type or does not have a mask.
///
/// # Examples
/// ```rust
/// use libxivdat::dat_type::{DATType, get_mask_for_type};
///
/// let mask = get_mask_for_type(&DATType::Macro).unwrap();
/// # let mut raw_macro_bytes = [0u8; 1];
/// for byte in raw_macro_bytes.iter_mut() {
///    *byte = *byte ^ mask;
/// }
/// ```
pub fn get_mask_for_type(file_type: &DATType) -> Option<u8> {
    match file_type {
        DATType::Gearset
        | DATType::GoldSaucer
        | DATType::ItemFinder
        | DATType::ItemOrder
        | DATType::Keybind
        | DATType::Macro
        | DATType::RecentTells => Some(0x73),
        DATType::Hotbar | DATType::UISave => Some(0x31),
        DATType::LogFilter => Some(0x00),
        _ => None,
    }
}

/// Gets the default header ending byte for a given DAT type.
/// The purpose of this value is unknown, but it is a fixed value based on file type.
/// Returns `None` if the file is of unknown type.
///
/// # Examples
/// ```rust
/// use libxivdat::dat_type::{DATType, get_default_end_byte_for_type};
/// let end_byte = get_default_end_byte_for_type(&DATType::Macro);
/// ```
pub fn get_default_end_byte_for_type(file_type: &DATType) -> Option<u8> {
    match file_type {
        DATType::Gearset
        | DATType::GoldSaucer
        | DATType::ItemFinder
        | DATType::ItemOrder
        | DATType::Keybind
        | DATType::Macro
        | DATType::RecentTells => Some(0xFF),
        DATType::Hotbar => Some(0x31),
        DATType::LogFilter => Some(0x00),
        DATType::UISave => Some(0x21),
        _ => None,
    }
}

/// Gets the default maximum content size of a DAT file for a given type.
/// Returns `None` if the file is of unknown type or has no standard size.
///
/// # Examples
/// ```rust
/// use libxivdat::dat_type::{DATType, get_default_max_size_for_type};
/// let max_size = get_default_max_size_for_type(&DATType::Macro).unwrap();
/// ```
pub fn get_default_max_size_for_type(file_type: &DATType) -> Option<u32> {
    get_default_max_size_for_type_and_version(file_type, get_default_file_version(file_type))
}

/// Gets the default maximum content size of a DAT file for a given type.
/// Returns `None` if the file is of unknown type or has no standard size.
///
/// # Examples
/// ```rust
/// use libxivdat::dat_type::{DATType, get_default_max_size_for_type_and_version};
/// let max_size = get_default_max_size_for_type_and_version(&DATType::Macro, 2).unwrap();
/// ```
pub fn get_default_max_size_for_type_and_version(file_type: &DATType, file_version: u16) -> Option<u32> {
    match (file_type, file_version) {
        (DATType::Gearset, 0x6A) => Some(45253),
        (DATType::Gearset, 0x6B) => Some(44849),
        (DATType::Gearset, 0x6C) => Some(45253),
        (DATType::Gearset, 0x6D) => Some(45657),

        (DATType::GoldSaucer, 0x66) => Some(629),
        (DATType::GoldSaucer, 0x67) => Some(649),

        (DATType::Hotbar, 0x04) => Some(204800),

        (DATType::ItemFinder, 0xC8) => Some(12411),
        (DATType::ItemFinder, 0xC9) => Some(13219),
        (DATType::ItemFinder, 0xCA) => Some(14030),

        (DATType::ItemOrder, 0x67) => Some(15193),
        (DATType::ItemOrder, 0x68) => Some(15103),

        (DATType::Keybind, 0x65) => Some(20480),

        (DATType::LogFilter, 0x02) => Some(2048),
        (DATType::LogFilter, 0x03) => Some(2048),

        (DATType::Macro, 0x02) => Some(286720),

        (DATType::RecentTells, 0x64) => Some(2048),

        (DATType::UISave, 0x01) => Some(64512),
        _ => None,
    }
}

/// Return a "default" file version for a DAT file type.
///
/// File versions are in flux and may be incremented by later patches to the game. The versions returned
/// here are the versions used in test data at the time of libxivdat's development (Patch 5.5) and may not
/// correspond to the current release.
pub fn get_default_file_version(dat_type: &DATType) -> u16 {
    match dat_type {
        DATType::Gearset => 0x6B,
        DATType::GoldSaucer => 0x67,
        DATType::Hotbar => 0x04,
        DATType::ItemFinder => 0xCA,
        DATType::ItemOrder => 0x67,
        DATType::Keybind => 0x65,
        DATType::LogFilter => 0x03,
        DATType::Macro => 0x02,
        DATType::RecentTells => 0x64,
        DATType::UISave => 0x01,
        _ => 0x00,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    const FILE_TYPE_MAP: [(DATType, &str); 9] = [
        (DATType::ACQ, "./resources/default_dats/ACQ.DAT"),
        (DATType::GEARSET, "./resources/default_dats/GEARSET.DAT"),
        (DATType::GS, "./resources/default_dats/GS.DAT"),
        (DATType::ITEMFDR, "./resources/default_dats/ITEMFDR.DAT"),
        (DATType::ITEMODR, "./resources/default_dats/ITEMODR.DAT"),
        (DATType::KEYBIND, "./resources/default_dats/KEYBIND.DAT"),
        (DATType::LOGFLTR, "./resources/default_dats/LOGFLTR.DAT"),
        (DATType::MACRO, "./resources/default_dats/MACRO.DAT"),
        (DATType::UISAVE, "./resources/default_dats/UISAVE.DAT"),
    ];

    const VERSIONED_FILE_MAP: [(DATType, &str, u16); 17] = [
        (DATType::ACQ, "./resources/versioned/ACQ_64.DAT", 0x64),
        (DATType::GEARSET, "./resources/versioned/GEARSET_6A.DAT", 0x6A),
        (DATType::GEARSET, "./resources/versioned/GEARSET_6B.DAT", 0x6B),
        (DATType::GEARSET, "./resources/versioned/GEARSET_6C.DAT", 0x6C),
        (DATType::GEARSET, "./resources/versioned/GEARSET_6D.DAT", 0x6D),
        (DATType::GS, "./resources/versioned/GS_66.DAT", 0x66),
        (DATType::GS, "./resources/versioned/GS_67.DAT", 0x67),
        (DATType::ITEMFDR, "./resources/versioned/ITEMFDR_C8.DAT", 0xC8),
        (DATType::ITEMFDR, "./resources/versioned/ITEMFDR_C9.DAT", 0xC9),
        (DATType::ITEMFDR, "./resources/versioned/ITEMFDR_CA.DAT", 0xCA),
        (DATType::ITEMODR, "./resources/versioned/ITEMODR_67.DAT", 0x67),
        (DATType::ITEMODR, "./resources/versioned/ITEMODR_68.DAT", 0x68),
        (DATType::KEYBIND, "./resources/versioned/KEYBIND_65.DAT", 0x65),
        (DATType::LOGFLTR, "./resources/versioned/LOGFLTR_02.DAT", 0x02),
        (DATType::LOGFLTR, "./resources/versioned/LOGFLTR_03.DAT", 0x03),
        (DATType::MACRO, "./resources/versioned/MACRO_02.DAT", 0x02),
        (DATType::UISAVE, "./resources/versioned/UISAVE_01.DAT", 0x01),
    ];

    #[test]
    fn test_from_header_bytes() -> Result<(), String> {
        for case in FILE_TYPE_MAP.iter() {
            let mut file = match File::open(case.1) {
                Ok(file) => file,
                Err(err) => return Err(format!("Error opening file: {}", err)),
            };
            let mut buf = [0u8; 2];
            match file.read(&mut buf) {
                Ok(_) => (),
                Err(err) => return Err(format!("Error reading file: {}", err)),
            };
            let id_bytes = u16::from_le_bytes(buf);
            assert_eq!(DATType::from(id_bytes as u32), case.0);
        }
        Ok(())
    }

    #[test]
    fn test_get_default_end_byte_for_type() -> Result<(), String> {
        for case in FILE_TYPE_MAP.iter() {
            match get_default_end_byte_for_type(&case.0) {
                Some(_) => (),
                None => return Err(format!("No value returned for case {}.", case.1)),
            };
        }
        Ok(())
    }

    #[test]
    fn test_get_default_max_size_for_type() -> Result<(), String> {
        for case in FILE_TYPE_MAP.iter() {
            match get_default_max_size_for_type(&case.0) {
                Some(_) => (),
                None => return Err(format!("No value returned for case {}.", case.1)),
            };
        }
        Ok(())
    }

    #[test]
    fn test_get_default_max_size_for_type_and_version() -> Result<(), String> {
        for case in VERSIONED_FILE_MAP.iter() {
            match get_default_max_size_for_type_and_version(&case.0, case.2) {
                Some(_) => (),
                None => return Err(format!("No value returned for case {}.", case.1)),
            };
        }
        Ok(())
    }

    #[test]
    fn test_get_mask_for_type() -> Result<(), String> {
        for case in FILE_TYPE_MAP.iter() {
            match get_mask_for_type(&case.0) {
                Some(_) => (),
                None => return Err(format!("No value returned for case {}.", case.1)),
            };
        }
        Ok(())
    }
}
