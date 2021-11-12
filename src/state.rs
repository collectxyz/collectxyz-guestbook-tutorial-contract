use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use collectxyz::nft::Coordinates;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

pub const OWNER: Item<String> = Item::new("owner");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The address of the xyz NFT contract.
    pub xyz_nft_contract: Addr,
    /// The character limit of each guestbook entry, e.g., 240.
    pub character_limit: u32,
    /// The fee required to leave an entry in a guestbook.
    pub entry_fee: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Entry {
    /// The xyz token ID associated with this guestbook entry.
    pub author_xyz_id: String,
    /// The coordinate location associated with this guestbook entry.
    pub coordinates: Coordinates,
    /// The text content of the guestbook entry.
    pub text: String,
}

// Build a composite primary key from an xyz token ID and a set of coordinates.
pub fn entries_key(author_xyz_id: String, coordinates: Coordinates) -> Vec<u8> {
    vec![author_xyz_id.as_bytes(), &coordinates.to_bytes()].concat()
}

// Define storage multiindexes to make it easier to load all guestbook entries associated with
// a given xyz ID or a given set of coordinates.
pub struct EntryIndexes<'a> {
    pub author_xyz_id: MultiIndex<'a, (String, Vec<u8>), Entry>,
    pub coordinates: MultiIndex<'a, (Vec<u8>, Vec<u8>), Entry>,
}

impl<'a> IndexList<Entry> for EntryIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Entry>> + '_> {
        let v: Vec<&dyn Index<Entry>> = vec![&self.author_xyz_id, &self.coordinates];
        Box::new(v.into_iter())
    }
}

// Build and return the indexed map of guestbook entries for use in contract handlers.
pub fn entries<'a>() -> IndexedMap<'a, &'a [u8], Entry, EntryIndexes<'a>> {
    let indexes = EntryIndexes {
        author_xyz_id: MultiIndex::new(
            |n: &Entry, k: Vec<u8>| (n.author_xyz_id.clone(), k),
            "entries",
            "entries__author_xyz_id",
        ),
        coordinates: MultiIndex::new(
            |n: &Entry, k: Vec<u8>| (n.coordinates.to_bytes(), k),
            "entries",
            "entries__coordinates",
        ),
    };
    IndexedMap::new("entries", indexes)
}
