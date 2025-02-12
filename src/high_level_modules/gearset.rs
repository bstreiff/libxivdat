use crate::dat_error::DATError;
use crate::dat_file::DATFile;
use crate::dat_type::DATType;
use crate::high_level::Validate;
use std::io::{Seek, SeekFrom, Read};
use std::convert::TryInto;
use std::ffi::CStr;

/// The number of [`Gearset`] items expected in a valid macro file.
pub const EXPECTED_ITEM_COUNT: usize = 101;

pub const EQUIPMENT_SLOT_COUNT: usize = 14;

/// Named indices for equipment slots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentSlot {
    /// Primary (Weapon) equipment slot.
    Primary = 0,
    /// Secondary (Offhand) equipment slot.
    Secondary = 1,
    /// Head equipment slot.
    Head = 2,
    /// Body equipment slot.
    Body = 3,
    /// Hands equipment slot.
    Hands = 4,
    /// Belt equipment slot (deprecated since Patch 6.0)
    Belt = 5,
    /// Legs equipment slot.
    Legs = 6,
    /// Feet equipment slot.
    Feet = 7,
    /// Earrings equipment slot.
    Earrings = 8,
    /// Necklace equipment slot.
    Necklace = 9,
    /// Bracelet equipment slot.
    Bracelet = 10,
    /// Left ring equipment slot.
    RingLeft = 11,
    /// Right right equipment slot.
    RingRight = 12,
    /// Soul crystal equipment slot.
    SoulCrystal = 13,
}

/// Maximum number of materia slots.
pub const MATERIA_SLOT_COUNT: usize = 5;

/// Oldest supported gearset file version for this implementation.
pub const SUPPORTED_VERSION_MIN: u16 = 0x6A;

/// Newest supported gearset file version for this implementation.
pub const SUPPORTED_VERSION_MAX: u16 = 0x6D;

/// Resource definition for a Final Fantasy XIV gearset list.
///
/// A gearset list consists of 101 different gearsets (100 saved, plus one for what is currently equipped).
/// Additionally, there is a marker to indicate which gearset is considered "active" (e.g. if the current
/// one is saved, what gearset does it overwrite).
///
/// Gearset information uses an extensive amount of identification numbers, such as for jobs or for items.
/// Since this information is added to for nearly every patch, this library does not attempt to provide
/// `enum` types or similar for getting this information; consider using [XIVAPI](https://xivapi.com) or
/// [ironworks](https://crates.io/crates/ironworks) for associating this information with game data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GearsetList {
    /// Gearset format version.
    pub version: u16,

    /// Active gearset index.
    pub active: u8,

    /// Vector of gearsets.
    pub gearsets: Vec<Gearset>,
}

/// Resource definition for a Final Fantasy XIV gearset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gearset {
    /// The client-side index for this gearset. Gearsets are stored in incrementing order.
    pub index: u8,

    /// Name of this gearset, up to 15 UTF-8 characters in length.
    pub name: String,

    /// Class/job for this gearset. The identifiers can be looked up in game data using the
    /// [ClassJob](https://github.com/xivapi/ffxiv-datamining/blob/add2f2e3acd17e3442f2fd1013fa7b70afad7504/csv/ClassJob.csv) table.
    pub class_job: u8,

    /// Linked glamour plate. (Patch 4.3+)
    pub glamour_plate: u8,

    /// Average item level. (Patch 6.1+)
    pub average_item_level: u16,

    /// Server-side index for this gearset.
    pub server_index: u8,

    /// Equipment.
    pub equipment: Vec<EquipmentItem>,

    /// Facewear. (Patch 7.0+) The identifiers can be looked up in game data using the
    /// [Glasses](https://github.com/xivapi/ffxiv-datamining/blob/add2f2e3acd17e3442f2fd1013fa7b70afad7504/csv/Glasses.csv) table.
    pub facewear_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentItem {
    /// Item identifier for the equipment in this slot.
    pub item_id: u32,

    /// Item identifier for the glamour over this item, if applicable.
    pub glamour_item_id: u32,

    /// Primary dye channel color for this item.
    pub dye_primary: u8,

    /// Secondary dye channel color for this item. (Patch 7.0+)
    pub dye_secondary: u8,

    /// Materia type identifier. This can be looked up using the [Materia](https://github.com/xivapi/ffxiv-datamining/blob/add2f2e3acd17e3442f2fd1013fa7b70afad7504/csv/Materia.csv) table
    /// in order to correlate it to a materia and its stats bonuses. This also applies to weapons with customizable substats (relics),
    /// which implement those substats as invisible materia.
    pub materia_types: [u16; MATERIA_SLOT_COUNT],

    /// Materia grade.
    pub materia_grades: [u8; MATERIA_SLOT_COUNT],
}


impl Validate for GearsetList {
    fn validate(&self) -> Option<DATError> {
        if self.gearsets.len() < 101 {
            return Some(DATError::Underflow("Gearset list has fewer than 101 gearsets."));
        }
        if self.gearsets.len() > 101 {
            return Some(DATError::Underflow("Gearset list has greater than 101 gearsets."));
        }
        if (self.active as usize) > (self.gearsets.len() - 1) {
            return Some(DATError::Overflow("Active gearset is out-of-range."));
        }
        None
    }
}

impl GearsetList {
    pub fn new(active: u8, gearsets: Vec<Gearset>) -> Result<GearsetList, DATError> {
        let res_gslist = GearsetList {
            version: SUPPORTED_VERSION_MAX,
            active: active,
            gearsets: gearsets.clone(),
        };
        match res_gslist.validate() {
            Some(err) => Err(err),
            None => Ok(res_gslist),
        }
    }
}

pub fn read_gearset(dat_file: &mut DATFile) -> Result<GearsetList, DATError> {
    if dat_file.file_type() != DATType::Gearset {
        Err(DATError::IncorrectType(
            "Attempted to read a gearset list from a non-gearset file.",
        ))
    } else if dat_file.file_version() < SUPPORTED_VERSION_MIN {
        Err(DATError::UnsupportedFileVersion(
            "Gearset file is too old to be supported by this implementation",
        ))
    } else if dat_file.file_version() > SUPPORTED_VERSION_MAX {
        Err(DATError::UnsupportedFileVersion(
            "Gearset file is too new to be supported by this implementation",
        ))
    } else {
        Ok(read_gearsetlist_unsafe(dat_file)?)
    }
}

pub fn read_gearsetlist_unsafe(dat_file: &mut DATFile) -> Result<GearsetList, DATError> {
    // First byte is unknown, skip
    dat_file.seek(SeekFrom::Current(1))?;

    let mut active_bytes = [0u8; 1];
    match dat_file.read_exact(&mut active_bytes) {
        Ok(_) => (),
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(DATError::EndOfFile("Found EOF looking for next section."))
        }
        Err(err) => return Err(DATError::from(err)),
    };

    let active = u8::from_le_bytes(active_bytes);

    // Third and fourth bytes are unknown, skip
    dat_file.seek(SeekFrom::Current(2))?;

    let mut gearset_vec = Vec::<Gearset>::with_capacity(EXPECTED_ITEM_COUNT);
    for _ in 1..EXPECTED_ITEM_COUNT {
        gearset_vec.push(read_gearset_unsafe(dat_file)?);
    }

    Ok(GearsetList {
        version: dat_file.file_version(),
        active: u8::from_le_bytes(active_bytes),
        gearsets: gearset_vec,
    })
}

pub fn read_gearset_unsafe(dat_file: &mut DATFile) -> Result<Gearset, DATError> {
    let mut index_byte = [0u8; 1];
    let mut name_bytes = [0u8; 48];
    let mut class_job_byte = [0u8; 1];
    let mut glamour_plate_byte = [0u8; 1];
    let mut average_item_level_bytes = [0u8; 2];
    let mut server_index_byte = [0u8; 1];
    let mut flags_byte = [0u8; 1];
    let mut facewear_id_bytes = [0u8; 4];

    dat_file.read_exact(&mut index_byte)?;
    dat_file.read_exact(&mut name_bytes)?;
    dat_file.read_exact(&mut class_job_byte)?;

    // Patch 4.3 added glamour plate linking.
    // Patch 6.1 removed item level out of the gear set name and into its own field.
    if dat_file.file_version() >= 0x6C {
        dat_file.read_exact(&mut glamour_plate_byte)?;
        dat_file.seek(SeekFrom::Current(1))?;
        dat_file.read_exact(&mut average_item_level_bytes)?;
    }

    dat_file.read_exact(&mut server_index_byte)?;
    dat_file.read_exact(&mut flags_byte)?;

    let mut equipment = Vec::<EquipmentItem>::with_capacity(EQUIPMENT_SLOT_COUNT);
    for _ in 0..EQUIPMENT_SLOT_COUNT {
        let mut item_id_bytes = [0u8; 4];
        let mut glamour_item_id_bytes = [0u8; 4];
        let mut dye_primary_byte = [0u8; 1];
        let mut dye_secondary_byte = [0u8; 1];
        let mut materia_types_bytes = [0u8; MATERIA_SLOT_COUNT * 2];
        let mut materia_grades_bytes = [0u8; MATERIA_SLOT_COUNT];

        dat_file.read_exact(&mut item_id_bytes)?;
        dat_file.read_exact(&mut glamour_item_id_bytes)?;
        dat_file.read_exact(&mut dye_primary_byte)?;
        dat_file.read_exact(&mut dye_secondary_byte)?;
        dat_file.read_exact(&mut materia_types_bytes)?;
        dat_file.read_exact(&mut materia_grades_bytes)?;
        // three bytes of padding
        dat_file.seek(SeekFrom::Current(3))?;

        // convert from byte array to u16le array
        let mut materia_types = [0u16; 5];
        for (chunk, o) in materia_types_bytes.chunks_exact(2).zip(materia_types.iter_mut()) {
            *o = u16::from_le_bytes(chunk.try_into().unwrap());
	}

        equipment.push(EquipmentItem {
            item_id: u32::from_le_bytes(item_id_bytes),
            glamour_item_id: u32::from_le_bytes(glamour_item_id_bytes),
            dye_primary: u8::from_le_bytes(dye_primary_byte),
            dye_secondary: u8::from_le_bytes(dye_secondary_byte),
            materia_types: materia_types,
            materia_grades: materia_grades_bytes,
        });
    }

    // Patch 7.0 (0x6D) added Facewear.
    if dat_file.file_version() >= 0x6D {
        dat_file.read_exact(&mut facewear_id_bytes)?;
    }

    Ok(Gearset {
        index: u8::from_le_bytes(index_byte),
        //name: str::from_utf8(&name_bytes)?.to_string(),
        name: CStr::from_bytes_until_nul(&name_bytes).unwrap().to_str()?.to_owned(),
        class_job: u8::from_le_bytes(class_job_byte),
        glamour_plate: u8::from_le_bytes(glamour_plate_byte),
        average_item_level: u16::from_le_bytes(average_item_level_bytes),
        server_index: u8::from_le_bytes(server_index_byte),
        equipment: equipment,
        facewear_id: u32::from_le_bytes(facewear_id_bytes),
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    //use crate::dat_file::read_content;

    // This test file is from no earlier than 4.2 (has Byakko's Stone Sword in an equipment slot)
    const TEST_FILE_PATH_6A: &str = "./resources/versioned/GEARSET_6A.DAT";
    // This test file is from no earlier than 5.5 (has Diamond Zeta Chakrams in an equipment slot)
    const TEST_FILE_PATH_6B: &str = "./resources/versioned/GEARSET_6B.DAT";
    // It is unclear what version this test file is from based on the gear it contains.
    const TEST_FILE_PATH_6C: &str = "./resources/versioned/GEARSET_6C.DAT";
    // This test file is from 7.1.
    const TEST_FILE_PATH_6D: &str = "./resources/versioned/GEARSET_6D.DAT";

    #[test]
    fn test_read_6a() -> Result<(), String> {
        let mut dat_file = match DATFile::open(TEST_FILE_PATH_6A) {
            Ok(dat_file) => dat_file,
            Err(err) => return Err(format!("Error opening file: {}", err)),
        };
        let a_gearset = match read_gearset(&mut dat_file) {
            Ok(a_gearset) => a_gearset,
            Err(err) => return Err(format!("Error reading test gearset: {}", err)),
        };

        const CLASS_JOB_MONK: u8 = 20;
        const ITEM_SPHAIRAI_ATMA: u32 = 7825;

        // read some data out of it.
        if a_gearset.gearsets[6].class_job != CLASS_JOB_MONK {
            return Err(format!("Error reading test gearset: Unexpected class/job for set 6"));
        }
        if a_gearset.gearsets[6].equipment[EquipmentSlot::Primary as usize].item_id != ITEM_SPHAIRAI_ATMA {
            return Err(format!("Error reading test gearset: Unexpected primary item for set 6"));
        }

        Ok(())
    }

    #[test]
    fn test_read_6b() -> Result<(), String> {
        let mut dat_file = match DATFile::open(TEST_FILE_PATH_6B) {
            Ok(dat_file) => dat_file,
            Err(err) => return Err(format!("Error opening file: {}", err)),
        };
        let a_gearset = match read_gearset(&mut dat_file) {
            Ok(a_gearset) => a_gearset,
            Err(err) => return Err(format!("Error reading test gearset: {}", err)),
        };

        const CLASS_JOB_DARK_KNIGHT: u8 = 32;
        const ITEM_WOEBORN: u32 = 30234;

        // read some data out of it.
        if a_gearset.gearsets[6].class_job != CLASS_JOB_DARK_KNIGHT {
            return Err(format!("Error reading test gearset: Unexpected class/job for set 6"));
        }
        if a_gearset.gearsets[6].equipment[EquipmentSlot::Primary as usize].item_id != ITEM_WOEBORN {
            return Err(format!("Error reading test gearset: Unexpected primary item for set 6"));
        }

        Ok(())
    }

    #[test]
    fn test_read_6c() -> Result<(), String> {
        let mut dat_file = match DATFile::open(TEST_FILE_PATH_6C) {
            Ok(dat_file) => dat_file,
            Err(err) => return Err(format!("Error opening file: {}", err)),
        };
        let a_gearset = match read_gearset(&mut dat_file) {
            Ok(a_gearset) => a_gearset,
            Err(err) => return Err(format!("Error reading test gearset: {}", err)),
        };

        const CLASS_JOB_GUNBREAKER: u8 = 37;
        const ITEM_HIGH_STEEL_GUNBLADE_HQ: u32 = 1027357;

        // read some data out of it.
        if a_gearset.gearsets[3].class_job != CLASS_JOB_GUNBREAKER {
            return Err(format!("Error reading test gearset: Unexpected class/job for set 3"));
        }
        if a_gearset.gearsets[3].average_item_level != 186 {
            return Err(format!("Error reading test gearset: Unexpected average item level for set 3"));
        }
        if a_gearset.gearsets[3].equipment[EquipmentSlot::Primary as usize].item_id != ITEM_HIGH_STEEL_GUNBLADE_HQ {
            return Err(format!("Error reading test gearset: Unexpected primary item for set 3"));
        }

        Ok(())
    }

    #[test]
    fn test_read_6d() -> Result<(), String> {
        let mut dat_file = match DATFile::open(TEST_FILE_PATH_6D) {
            Ok(dat_file) => dat_file,
            Err(err) => return Err(format!("Error opening file: {}", err)),
        };
        let a_gearset = match read_gearset(&mut dat_file) {
            Ok(a_gearset) => a_gearset,
            Err(err) => return Err(format!("Error reading test gearset: {}", err)),
        };

        const CLASS_JOB_REAPER: u8 = 39;
        const ITEM_AUGMENTED_CRYPTLURKER_WAR_SCYTHE: u32 = 35771;

        // read some data out of it.
        if a_gearset.gearsets[8].class_job != CLASS_JOB_REAPER {
            return Err(format!("Error reading test gearset: Unexpected class/job for set 8"));
        }
        if a_gearset.gearsets[8].average_item_level != 530 {
            return Err(format!("Error reading test gearset: Unexpected average item level for set 8"));
        }
        if a_gearset.gearsets[8].equipment[EquipmentSlot::Primary as usize].item_id != ITEM_AUGMENTED_CRYPTLURKER_WAR_SCYTHE {
            return Err(format!("Error reading test gearset: Unexpected primary item for set 8"));
        }

        Ok(())
    }

}
