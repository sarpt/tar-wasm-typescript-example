#![allow(unused)]

use std::{io::{BufReader, ErrorKind, Read}};

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;

const TAR_HEADER_LEN: usize = 512;
const FILENAME_LEN: usize = 100;

const FILE_SIZE_OFFSET: usize = 124;
const FILE_SIZE_LEN: usize = 12;

const FILE_ENTRY_PADDING_BASE: usize = 512;

fn main() {
  use wasm_bindgen::prelude::*;

  #[wasm_bindgen]
  pub fn parse_tar(payload: &[u8]) -> Result<Tar, TarErrors> {
    if payload.len() < TAR_HEADER_LEN {
      return Err(TarErrors::NotATarFile);
    }

    let mut entries: Vec<Entry> = Vec::new();
    let mut entry_offset = 0;
    loop {
      console::log_1(&format!("[WASM] entry offset: {entry_offset}", ).into());

      if entry_offset >= payload.len() {
          break;
      }

      if entry_offset + TAR_HEADER_LEN >= payload.len() {
          return Err(TarErrors::DamagedHeader);
      }


      let filename =
          String::from_utf8_lossy(&payload[entry_offset..entry_offset + FILENAME_LEN]).trim_end_matches('\0').to_string();
      if filename.is_empty() {
        // tar files are ending with an empty heading (all zeroes), so assumption is that empty
        // filename equal empty header, hence end of file. may not be correct but that doesnt
        // matter here
        break;
      }

      let entry_data_size = usize::from_str_radix(
          String::from_utf8_lossy(
              &payload[entry_offset + FILE_SIZE_OFFSET
                  ..entry_offset + FILE_SIZE_OFFSET + FILE_SIZE_LEN],
          ).trim_end_matches('\0'),
          8,
      )
      .map_err(|_| TarErrors::SizeUnreadable)?;
      let payload: Vec<u8> = Vec::from(&payload[entry_offset + TAR_HEADER_LEN..entry_offset + TAR_HEADER_LEN + entry_data_size]);
      entries.push(Entry { variant: EntryType::NormalFile, filename, size: entry_data_size, payload });
      entry_offset += TAR_HEADER_LEN + entry_data_size.div_ceil(FILE_ENTRY_PADDING_BASE) * FILE_ENTRY_PADDING_BASE;
    }

    let tar = Tar {
        entries,
    };
    Ok(tar)
  }
}


#[wasm_bindgen]
pub struct Tar {
    entries: Vec<Entry>,
}

#[wasm_bindgen]
impl Tar {
  #[wasm_bindgen]
  pub fn get_filenames(&self) -> Vec<String> {
    self.entries.iter().map(|entry| entry.filename.clone()).collect()  
  }

  #[wasm_bindgen]
  pub fn get_payload(&self, name: &str) -> Result<Vec<u8>, TarErrors> {
    match self.entries.iter().find(|entry| entry.filename == name) {
      Some(entry) => Ok(entry.payload.clone()),
      None => Err(TarErrors::FileNotFound),
    }
  }
}

pub struct Entry {
    variant: EntryType,
    filename: String,
    size: usize,
    payload: Vec<u8>,
}

#[derive(Copy, Clone)]
pub enum EntryType {
    NormalFile,
    HardLink,
    SymbolicLink,
    CharacterSpecial,
    BlockSpecial,
    Directory,
    FIFO,
    Contagious,
    GlobalExtenderHeaderMetadata,
    NextFileExtenderHeaderMetadata,
    VendorSpecificExtension,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum TarErrors {
    NotATarFile = "Not a tar file",
    DamagedHeader = "File header could not be read",
    SizeUnreadable = "Could not read file entry size",
    FileNotFound = "File with provided filename could not be found",
}
