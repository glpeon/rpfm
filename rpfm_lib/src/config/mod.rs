//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with configuration folder stuff.

This module contains all the code related with the configuration folder stuff. This means here is the code to properly initialize and return the config folder.
Many things depend on being able to read and write files in that folder, so always remember to initialize it on start, and stop if the initialization failed.
!*/

use directories::ProjectDirs;
use rpfm_error::{ErrorKind, Result};
use std::path::PathBuf;
use std::fs::DirBuilder;

/// Qualifier for the config folder. Only affects MacOS.
const QUALIFIER: &str = "";

/// Organisation for the config folder. Only affects Windows and MacOS.
const ORGANISATION: &str = "";

/// Name of the config folder.
const PROGRAM_NAME: &str = "Rusted PackFile Manager";

/// Function to initialize the config folder, so RPFM can use it to store his stuff.
///
/// This can fail, so if this fails, better stop the program and check why it failed.
#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {
	match ProjectDirs::from(&QUALIFIER, &ORGANISATION, &PROGRAM_NAME) {
		Some(proj_dirs) => {
			let config_path = proj_dirs.config_dir();
	        DirBuilder::new().recursive(true).create(&config_path)?;
	        if config_path.is_dir() { Ok(()) }
            else { Err(ErrorKind::IOFolderCannotBeOpened)? }
		},
		None => Err(ErrorKind::IOFolderCannotBeOpened)?
	}
}

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn get_config_path() -> Result<PathBuf> {
	if cfg!(debug_assertions) { std::env::current_dir().map_err(|x| From::from(x)) } else {
		match ProjectDirs::from(&QUALIFIER, &ORGANISATION, &PROGRAM_NAME) {
			Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
			None => Err(ErrorKind::IOFolderCannotBeOpened)?
		}
	}
}
