use std::{env::current_exe, fs, path::Path, sync::Arc};
use ironworks::{excel::{Excel, Language, Sheet}, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{exdschema::{Provider, Version}, Schema};
use crate::err::{Err, ToUnknownErr};

use super::Args;

pub(crate) struct Init<'a> {
    pub excel: Excel,
    pub sheet: Sheet<&'a str>,
    pub version: Version,
    pub schema: ironworks_schema::Sheet
}

impl <'a> Init<'a> {
    pub fn new(sheet_name: &'static str, args: &Args<impl std::io::Write>) -> Result<Self, Err> {
        let game_resource = get_game_resource(&args.game_path.as_deref())?;
        let version = game_resource.version(0).unwrap();
    
        let ironworks = Arc::new(Ironworks::new().with_resource(SqPack::new(game_resource)));
        let excel = Excel::new(ironworks).with_default_language(Language::English);
        let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
    
        let (schema, version) = Self::get_schema(sheet_name, &version, args.refresh)?;
    
        Ok(Self { excel, sheet, schema, version })
    }

    pub fn get_schema(sheet_name: &str, version: &str, refresh: bool) -> Result<(ironworks_schema::Sheet, Version), Err> {
        let repository_directory = current_exe().ok().to_unknown_err()?.parent().to_unknown_err()?.join(format!("exdschema_{}", version));

        if refresh && repository_directory.exists() {
            fs::remove_dir_all(&repository_directory).to_unknown_err()?;
        }

        let provider = Provider::with().directory(repository_directory).build().to_unknown_err()?;
        let specifier = provider.specifier("HEAD", version).to_unknown_err()?;
        let version = provider.version(specifier).map_err(|_| Err::VersionNotFound(version.to_owned()))?;
        let schema = version.sheet(sheet_name).to_unknown_err()?;

        Ok((schema, version))
    }
}

pub fn get_game_resource(game_dir: &Option<&Path>) -> Result<Install, Err> {
    let game_resource = if let Some(game_dir) = game_dir {
        Some(Install::at(game_dir))
    } else {
        Install::search()
    }.ok_or(Err::GameNotFound)?;

    // There's currently an error in ironworks where search() always returns
    // Some(), even if no path was found. We do this check to ensure the path
    // actually points to the game.
    game_resource.version(0).map_err(|_| Err::GameNotFound)?;

    Ok(game_resource)
}
