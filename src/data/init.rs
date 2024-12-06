use std::{borrow::Cow, env::current_exe, fs, path::{Path, PathBuf}, sync::Arc};
use ironworks::{excel::{Excel, Language, Sheet, SheetIterator}, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{exdschema::{Provider, Version}, Node, Order, Schema};
use crate::err::{Err, ToUnknownErr};
use super::SheetColumn;

/// A builder for the main [`IronworksCli`] interface.
/// This is the entry point of the crate.
/// 
/// Note: you don't need to use [`IronworksBuilder`] if all
/// you want to do is extract an icon. Use [`crate::extract_icon()`]
/// instead.
#[derive(Debug, Clone, Default)]
pub struct IronworksBuilder {
    game_path: Option<PathBuf>,
    should_refresh_schema: bool
}

impl IronworksBuilder {
    /// Creates a new [`IronworksBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the fully quantified absolute path to Final Fantasy XIV's
    /// main directory. The passed directory should only contain a `boot`
    /// and `game` directory.
    ///
    /// If this function is not called, the CLI attempts to find the location
    /// of the game directory on the user's file system itself. This will
    /// likely fail if the game is installed on a different drive, or if the CLI
    /// is executed in WSL.
    pub fn game_path(&mut self, path: PathBuf) -> &mut Self {
        self.game_path = Some(path);

        self
    }

    /// Forcibly refreshes the cached EXDSchema, even if the upstream EXDSchema repository
    /// indicates that no new EXDSchema version has been published.
    ///
    /// Default is `false`, in which case the locally cached schema is only updated when the
    /// upstream repository publishes a new version of the schema.
    pub fn force_refresh(&mut self) -> &mut Self {
        self.should_refresh_schema = true;

        self
    }

    /// Builds an instance of the ironworks CLI.
    /// This function may be expensive to execute, as it will attempt to read
    /// or update the schema (if necessary) and find the FFXIV directory.
    pub fn build(self) -> Result<IronworksCli, Err> {
        let game_resource = get_game_resource(self.game_path.as_deref())?;
        let version_string = game_resource.version(0).unwrap();
        let ironworks = Arc::new(Ironworks::new().with_resource(SqPack::new(game_resource)));
        let excel = Excel::new(ironworks).with_default_language(Language::English);
        let schema = get_schema(&version_string, self.should_refresh_schema)?;

        Ok(IronworksCli { excel, schema, version: version_string })
    }
}

#[derive(Debug)]
pub(crate) struct SheetInfo<'a> {
    pub sheet: Sheet<&'a str>,
    pub schema: ironworks_schema::Sheet
}

/// A runtime instance of the ironworks interface.
/// Use [`IronworksBuilder`] to create an instance of this type.
#[derive(Debug)]
pub struct IronworksCli {
    excel: Excel,
    schema: Version,
    version: String
}

impl IronworksCli {
    pub(crate) fn get_sheet<'a>(&self, sheet_name: &'a str) -> Result<SheetInfo<'a>, Err> {
        Ok(SheetInfo {
            sheet: self.excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name.to_owned().into()))?,
            schema: self.schema.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name.to_owned().into()))?
        })
    }

    pub(crate) fn sheet_iter<'a>(&self, sheet_name: &'a str) -> Result<SheetIterator<&'a str>, Err> {
        Ok(self.excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name.to_owned().into()))?.into_iter())
    }

    /// Gets the game's version.
    pub fn version(&self) -> &str {
        &self.version
    }
}

fn get_schema(version: &str, refresh: bool) -> Result<Version, Err> {
    let repository_directory = current_exe().ok().to_unknown_err()?.parent().to_unknown_err()?.join(format!("exdschema_{}", version));

    if refresh && repository_directory.exists() {
        fs::remove_dir_all(&repository_directory).to_unknown_err()?;
    }

    let provider = Provider::with().directory(repository_directory).build().to_unknown_err()?;
    let specifier = provider.specifier("HEAD", version).to_unknown_err()?;
    let version = provider.version(specifier).map_err(|_| Err::VersionNotFound(version.to_owned()))?;

    Ok(version)
}

pub(crate) fn get_game_resource(game_dir: Option<&Path>) -> Result<Install, Err> {
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

impl <'a> SheetInfo<'a> {
    pub(crate) fn columns(&self) -> Result<impl Iterator<Item = SheetColumn> + use<'_>, Err> {
        self.filtered_columns_iter(None)
    }
    
    pub(crate) fn filtered_columns(&self, filter_columns: &'a [&'a str]) -> Result<impl Iterator<Item = SheetColumn> + use<'_>, Err> {
        self.filtered_columns_iter(Some(filter_columns))
    }

    fn filtered_columns_iter(&self, filter_columns: Option<&'a [&'a str]>) -> Result<impl Iterator<Item = SheetColumn> + use<'_>, Err> {
        let mut columns = self.sheet.columns().map_err(|_| Err::UnsupportedSheet(Cow::Owned(self.sheet.name())))?;
        let fields = if let Node::Struct(columns) = &self.schema.node { columns } else { Err(Err::UnsupportedSheet(Cow::Owned(self.sheet.name())))? };
    
        match self.schema.order {
            Order::Index => (),
            Order::Offset => columns.sort_by_key(|column| column.offset()),
        };
    
        Ok(fields.iter()
            .filter(move |x| {
                if let Some(filter_columns) = filter_columns {
                    filter_columns.contains(&x.name.as_ref())
                } else {
                    true
                }
            })
            .map(move |x| SheetColumn { name: x.name.clone(), column: columns[x.offset as usize].clone() }))
    }
}
