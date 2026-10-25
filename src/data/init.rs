use std::{borrow::Cow, env::current_exe, fs, io::{self, Cursor, Read}, path::{Path, PathBuf}, process, sync::Arc};
use ironworks::{excel::{Excel, Language, Sheet, SheetIterator}, sqpack::{Install, Location, Resource, SqPack}, Ironworks};
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
    should_refresh_schema: bool,
    requested_version: Option<String>
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
    pub fn game_path(mut self, path: PathBuf) -> Self {
        self.game_path = Some(path);

        self
    }

    /// Forcibly refreshes the cached EXDSchema, even if the upstream EXDSchema repository
    /// indicates that no new EXDSchema version has been published.
    ///
    /// Default is `false`, in which case the locally cached schema is only updated when the
    /// upstream repository publishes a new version of the schema.
    pub fn force_refresh(mut self) -> Self {
        self.should_refresh_schema = true;

        self
    }

    /// Forcibly refreshes the cached EXDSchema with the given EXDSChema version, even if the
    /// upstream EXDSchema repository indicates that no new EXDSchema version has been published.
    pub fn force_refresh_with_version(mut self, s: String) -> Self {
        self.should_refresh_schema = true;
        self.requested_version = Some(s);

        self
    }

    /// Builds an instance of the ironworks CLI.
    /// This function may be expensive to execute, as it will attempt to read
    /// or update the schema (if necessary) and find the FFXIV directory.
    pub fn build(self) -> Result<IronworksCli, Err> {
        let game_resource = get_game_resource(self.game_path.as_deref())?;
        let version_string = self.requested_version.unwrap_or_else(|| game_resource.version(0).unwrap());
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
    let repository_directory = current_exe().map_err(Err::IoError)?.parent().to_unknown_err(1)?.join(format!("exdschema_{}", version));

    if refresh && repository_directory.exists() {
        fs::remove_dir_all(&repository_directory).map_err(Err::IoError)?;
    }

    if !repository_directory.exists() {
        clone_schema(&repository_directory)?;
    }

    match open_schema(&repository_directory, version) {
        Ok(schema) => Ok(schema),
        // A broken cache (e.g. an empty repository left behind by an interrupted
        // clone) would otherwise fail every future run, so it is cloned again once.
        Err(_) => {
            discard_schema(&repository_directory)?;
            clone_schema(&repository_directory)?;
            open_schema(&repository_directory, version)
        }
    }
}

/// Opens the cloned EXDSchema repository at `directory` and resolves the schema
/// for the given game version (or the latest schema published before it).
fn open_schema(directory: &Path, version: &str) -> Result<Version, Err> {
    let provider = Provider::with().directory(directory).build().map_err(Err::SchemaError)?;
    let specifier = provider.specifier_v2_ver(version).map_err(Err::SchemaError)?;

    provider.version(specifier).map_err(|_| Err::VersionNotFound(version.to_owned()))
}

/// Clones the EXDSchema repository into `directory`.
///
/// The clone is made in a temporary sibling directory and only moved to `directory`
/// once it is complete, so that concurrently running processes never see a partial
/// clone. If another process moved its clone into place first, that one is kept.
fn clone_schema(directory: &Path) -> Result<(), Err> {
    let temp_directory = sibling_path(directory, "tmp");

    if temp_directory.exists() {
        fs::remove_dir_all(&temp_directory).map_err(Err::IoError)?;
    }

    if let Err(err) = Provider::with().directory(&temp_directory).build() {
        let _ = fs::remove_dir_all(&temp_directory);
        return Err(Err::SchemaError(err));
    }

    if let Err(err) = fs::rename(&temp_directory, directory) {
        fs::remove_dir_all(&temp_directory).map_err(Err::IoError)?;

        if !directory.exists() {
            return Err(Err::IoError(err));
        }
    }

    Ok(())
}

/// Removes the cached EXDSchema repository at `directory`.
///
/// The directory is first renamed so that concurrently running processes
/// don't try to delete the same files.
fn discard_schema(directory: &Path) -> Result<(), Err> {
    let discarded_directory = sibling_path(directory, "old");

    match fs::rename(directory, &discarded_directory) {
        Ok(()) => fs::remove_dir_all(&discarded_directory).map_err(Err::IoError),
        // Another process already discarded it.
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Err::IoError(err)),
    }
}

/// Returns a path next to `path` that is unique to this process.
fn sibling_path(path: &Path, tag: &str) -> PathBuf {
    let mut sibling = path.as_os_str().to_owned();
    sibling.push(format!(".{}{}", tag, process::id()));

    PathBuf::from(sibling)
}

/// Wraps [`Install`] so that each game file is read into memory in one go.
///
/// [`Install`] hands out a `BufReader` over the dat file, and ironworks seeks
/// to every block header with `SeekFrom::Start`, which discards the buffer.
/// That turns each 16-byte header into a full 8 KiB read, i.e. thousands of
/// reads for large sheets like `Action` (very slow on WSL's `/mnt` drives).
pub(crate) struct BufferedInstall(Install);

impl Resource for BufferedInstall {
    fn version(&self, repository: u8) -> Result<String, ironworks::Error> {
        self.0.version(repository)
    }

    type Index = <Install as Resource>::Index;
    fn index(&self, repository: u8, category: u8, chunk: u8) -> Result<Self::Index, ironworks::Error> {
        self.0.index(repository, category, chunk)
    }

    type Index2 = <Install as Resource>::Index2;
    fn index2(&self, repository: u8, category: u8, chunk: u8) -> Result<Self::Index2, ironworks::Error> {
        self.0.index2(repository, category, chunk)
    }

    type File = Cursor<Vec<u8>>;
    fn file(&self, repository: u8, category: u8, location: Location) -> Result<Self::File, ironworks::Error> {
        // The reader returned by Install is already limited to this file's range.
        let mut buffer = Vec::new();
        self.0.file(repository, category, location)?.read_to_end(&mut buffer)?;

        Ok(Cursor::new(buffer))
    }
}

pub(crate) fn get_game_resource(game_dir: Option<&Path>) -> Result<BufferedInstall, Err> {
    let game_resource = if let Some(game_dir) = game_dir {
        Some(Install::at(game_dir))
    } else {
        Install::search()
    }.ok_or(Err::GameNotFound)?;

    // There's currently an error in ironworks where search() always returns
    // Some(), even if no path was found. We do this check to ensure the path
    // actually points to the game.
    game_resource.version(0).map_err(|_| Err::GameNotFound)?;

    Ok(BufferedInstall(game_resource))
}

impl <'a> SheetInfo<'a> {
    pub(crate) fn columns(&self) -> Result<impl Iterator<Item = SheetColumn> + use<'_>, Err> {
        self.filtered_columns_iter(None)
    }
    
    pub(crate) fn filtered_columns(&self, filter_columns: &'a [&'a str]) -> Result<impl Iterator<Item = SheetColumn> + use<'_>, Err> {
        self.filtered_columns_iter(if filter_columns.is_empty() { None } else { Some(filter_columns) })
    }

    fn filtered_columns_iter(&self, filter_columns: Option<&'a [&'a str]>) -> Result<impl Iterator<Item = SheetColumn>, Err> {
        let mut columns = self.sheet.columns().map_err(|_| Err::UnsupportedSheet(Cow::Owned(self.sheet.name())))?;
        let fields = if let Node::Struct(columns) = &self.schema.node { columns } else { Err(Err::UnsupportedSheet(Cow::Owned(self.sheet.name())))? };
    
        match self.schema.order {
            Order::Index => (),
            Order::Offset => columns.sort_by_key(|column| column.offset),
        };
    
        Ok(fields.into_iter()
            .filter(move |x| {
                if let Some(filter_columns) = filter_columns {
                    filter_columns.contains(&x.name.as_ref())
                } else {
                    !x.name.contains("Unknown")
                }
            })
            .map(move |x| SheetColumn { name: x.name.clone(), column: columns[x.offset as usize].clone() }))
    }
}
