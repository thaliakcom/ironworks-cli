//! Reads an action's animation timeline file (`.tmb`): its length and when its hit goes out.
//!
//! A `.tmb` file is the magic `TMLB`, its size (`u32`) and its node count (`u32`), followed by its nodes.
//! Each node is a 4-byte tag and its size in bytes (`u32`, including the tag), followed by its data.
//! The `TMDH` node's data holds the timeline's duration as its third `u16`.
//! The layout follows physis' `tmb.rs` (https://docs.rs/crate/physis/0.6.0/source/src/tmb.rs).

use ironworks::Ironworks;
use super::IronworksCli;

/// The size of the file header: magic, size and node count.
const HEADER_SIZE: usize = 12;
/// The size of a node's header: tag and size.
const NODE_HEADER_SIZE: usize = 8;
/// Where the duration sits in the data of the `TMDH` node.
const DURATION_OFFSET: usize = 4;
/// The tag of the nodes that line up with an action's hit. What they are in the game's terms is
/// unknown; compared with fflogs, the hit comes about when the first one starts.
const HIT_TAG: &[u8] = b"C011";
/// Where a `C011` node's start frame sits in its data (after its `u16` ID).
const HIT_FRAME_OFFSET: usize = 2;

impl IronworksCli {
    /// The frame (at 30 frames per second) of the earliest `C011` node in the animation timeline with the
    /// given `ActionTimeline` key, which lines up with the action's hit, or [`None`] if the file doesn't exist
    /// or has no such node. Placeholders in the key (`[SKL_ID]`) must be filled in by the caller.
    pub fn timeline_hit_frame(&self, key: &str) -> Option<u16> {
        hit_frame_of(&read(&self.ironworks, key)?)
    }
}

/// The duration in the `TMDH` node of the timeline with the given `ActionTimeline` key,
/// or [`None`] if the file doesn't exist or has no such node.
pub(crate) fn timeline_duration(ironworks: &Ironworks, key: &str) -> Option<u16> {
    duration_of(&read(ironworks, key)?)
}

/// Reads the bytes of the timeline file with the given `ActionTimeline` key.
fn read(ironworks: &Ironworks, key: &str) -> Option<Vec<u8>> {
    ironworks.file::<Vec<u8>>(&format!("chara/action/{}.tmb", key)).ok()
}

/// Finds the duration in the `TMDH` node of a `.tmb` file's bytes.
fn duration_of(bytes: &[u8]) -> Option<u16> {
    let (_, data) = nodes(bytes)?.into_iter().find(|(tag, _)| *tag == b"TMDH")?;

    Some(u16::from_le_bytes(data.get(DURATION_OFFSET..DURATION_OFFSET + 2)?.try_into().ok()?))
}

/// Finds the earliest start frame of the `C011` nodes of a `.tmb` file's bytes.
fn hit_frame_of(bytes: &[u8]) -> Option<u16> {
    nodes(bytes)?.into_iter()
        .filter(|(tag, _)| *tag == HIT_TAG)
        .filter_map(|(_, data)| Some(u16::from_le_bytes(data.get(HIT_FRAME_OFFSET..HIT_FRAME_OFFSET + 2)?.try_into().ok()?)))
        .min()
}

/// Splits a `.tmb` file's bytes into its nodes' tags and data, or [`None`] if it isn't one or is truncated.
fn nodes(bytes: &[u8]) -> Option<Vec<(&[u8], &[u8])>> {
    if bytes.get(0..4)? != b"TMLB" {
        return None;
    }

    let node_count = u32::from_le_bytes(bytes.get(8..12)?.try_into().ok()?);
    let mut offset = HEADER_SIZE;
    let mut nodes = Vec::new();

    for _ in 0..node_count {
        let tag = bytes.get(offset..offset + 4)?;
        let size = u32::from_le_bytes(bytes.get(offset + 4..offset + 8)?.try_into().ok()?) as usize;

        if size < NODE_HEADER_SIZE {
            return None;
        }

        nodes.push((tag, bytes.get(offset + NODE_HEADER_SIZE..offset + size)?));
        offset += size;
    }

    Some(nodes)
}

#[cfg(test)]
mod tests {
    use super::{duration_of, hit_frame_of};

    /// A file with a `TMAL` node, a `TMDH` node whose duration is 90, and `C011` nodes at frames 40 and 24.
    fn sample() -> Vec<u8> {
        let mut bytes = b"TMLB".to_vec();
        bytes.extend_from_slice(&64u32.to_le_bytes());
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"TMAL");
        bytes.extend_from_slice(&12u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(b"TMDH");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 0, 0, 0, 90, 0, 3, 0]);
        bytes.extend_from_slice(b"C011");
        bytes.extend_from_slice(&12u32.to_le_bytes());
        bytes.extend_from_slice(&[2, 0, 40, 0]);
        bytes.extend_from_slice(b"C011");
        bytes.extend_from_slice(&12u32.to_le_bytes());
        bytes.extend_from_slice(&[3, 0, 24, 0]);
        bytes
    }

    #[test]
    fn reads_the_duration_after_other_nodes() {
        assert_eq!(duration_of(&sample()), Some(90));
    }

    #[test]
    fn rejects_other_files_and_truncated_ones() {
        assert_eq!(duration_of(b"XXXX"), None);
        assert_eq!(duration_of(&sample()[..30]), None);
    }

    #[test]
    fn reads_the_earliest_hit_frame() {
        assert_eq!(hit_frame_of(&sample()), Some(24));
        assert_eq!(hit_frame_of(&sample()[..40]), None);
    }
}
