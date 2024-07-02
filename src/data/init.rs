use std::{env::current_exe, fs, sync::Arc};
use clio::ClioPath;
use ironworks::{excel::{Excel, Language, Sheet}, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{saint_coinach::{Provider, Version}, Schema};
use crate::{cli::Cli, err::{Err, ToUnknownErr}};

pub struct Init<'a> {
    pub excel: Excel<'a>,
    pub sheet: Sheet<'a, &'a str>,
    pub version: Version,
    pub schema: ironworks_schema::Sheet
}

impl <'a> Init<'a> {
    pub fn new(sheet_name: &'static str, args: &Cli) -> Result<Self, Err> {
        let game_resource = Self::get_game_resource(&args.game)?;
        let version = game_resource.version(0).unwrap();
    
        let ironworks = Arc::new(Ironworks::new().with_resource(SqPack::new(game_resource)));
        let excel = Excel::with().language(Language::English).build(ironworks);
        let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
    
        let (schema, version) = Self::get_schema(sheet_name, &version, args.refresh)?;
    
        Ok(Self { excel, sheet, schema, version })
    }

    pub fn get_game_resource(game_dir: &Option<ClioPath>) -> Result<Install, Err> {
        let game_resource = if let Some(game_dir) = game_dir {
            Some(Install::at(game_dir.path()))
        } else {
            Install::search()
        }.ok_or(Err::GameNotFound)?;
    
        // There's currently an error in ironworks where search() always returns
        // Some(), even if no path was found. We do this check to ensure the path
        // actually points to the game.
        game_resource.version(0).map_err(|_| Err::GameNotFound)?;

        Ok(game_resource)
    }

    pub fn get_schema(sheet_name: &str, version: &str, refresh: bool) -> Result<(ironworks_schema::Sheet, Version), Err> {
        let repository_directory = current_exe().ok().to_unknown_err()?.parent().to_unknown_err()?.join(format!("saint_coinach_{}", version));

        if refresh && repository_directory.exists() {
            fs::remove_dir_all(&repository_directory).to_unknown_err()?;
        }

        let provider = Provider::with().directory(repository_directory).build().to_unknown_err()?;
        let version = provider.version("HEAD").map_err(|_| Err::VersionNotFound(version.to_owned()))?;
        let schema = version.sheet(sheet_name).to_unknown_err()?;

        Ok((schema, version))
    }
}
