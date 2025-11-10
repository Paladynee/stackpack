use core::mem;
use parking_lot::Mutex;
use std::{env, ffi::OsStr, mem::MaybeUninit, path::PathBuf, sync::LazyLock};

use anyhow::Result;
use libloading::Library;
use walkdir::WalkDir;

use crate::{
    mutator::Mutator,
    registered::{ALL_COMPRESSORS, RegisteredCompressor},
};

#[repr(C)]
pub struct FfiOption<T> {
    is_some: bool,
    payload: MaybeUninit<T>,
}

impl<T> FfiOption<T> {
    pub fn as_option(&self) -> Option<&T> {
        if self.is_some {
            // SAFETY: is_some being true guarantees that payload is initialized.
            Some(unsafe { self.payload.assume_init_ref() })
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct BoolFalseIfError {
    value: bool,
}

type FunctionSignature = unsafe extern "C" fn(
    data_ptr: *const u8,
    data_len: usize,
    vec_buf_ptr: *mut *mut u8,
    vec_len: *mut usize,
    vec_cap: *mut usize,
) -> BoolFalseIfError;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum APIError {
    MissingName,
    MissingDescription,
    MissingDriveMutation,
    MissingRevertMutation,
}

#[repr(C)]
pub struct StackpackPluginAPI {
    pub short_name: &'static str,
    pub description: FfiOption<&'static str>,
    pub drive_mutation: FunctionSignature,
    pub revert_mutation: FunctionSignature,
}

impl StackpackPluginAPI {
    pub unsafe fn from_library(lib: &Library) -> Result<Self, APIError> {
        unsafe {
            let short_name = lib
                .get::<*const &'static str>(b"STACKPACK_PLUGIN_SHORT_NAME\0")
                .map_err(|_| APIError::MissingName)?
                .read_unaligned();
            let description = lib
                .get::<*const FfiOption<&'static str>>(b"STACKPACK_PLUGIN_DESCRIPTION\0")
                .map_err(|_| APIError::MissingDescription)?
                .read_unaligned();
            let drive_mutation = lib
                .get::<FunctionSignature>(b"stackpack_plugin_drive_mutation\0")
                .map_err(|_| APIError::MissingDriveMutation)?;
            let revert_mutation = lib
                .get::<FunctionSignature>(b"stackpack_plugin_revert_mutation\0")
                .map_err(|_| APIError::MissingRevertMutation)?;
            Ok(StackpackPluginAPI {
                short_name,
                description,
                drive_mutation: *drive_mutation,
                revert_mutation: *revert_mutation,
            })
        }
    }
}

pub struct Plugin {
    pub loaded_from: PathBuf,
    pub api: StackpackPluginAPI,
    pub lib: Library,
}

impl Plugin {
    pub fn new(loaded_from: PathBuf, api: StackpackPluginAPI, lib: Library) -> Self {
        Plugin { loaded_from, api, lib }
    }
}

pub static LOADED_PLUGINS: LazyLock<Mutex<Vec<Plugin>>> = LazyLock::new(|| Mutex::new(vec![]));


pub unsafe fn load_plugins() {
    if_tracing! {{
        tracing::trace!(event = "loading_plugins");
    }}

    let path = match env::var_os("STACKPACK_PLUGINS_ROOT") {
        Some(t) => t,
        None => {
            if_tracing! {{
                tracing::info!(event = "no_plugins_path", "`STACKPACK_PLUGINS_ROOT` environment variable not set, skipping plugin loading");
            }};
            return;
        }
    };

    let mut pathbuf = PathBuf::from(path);
    pathbuf.push("plugins");

    if_tracing! {{
        tracing::debug!(event = "plugins", path = ?pathbuf.display(), "looking for plugins here");
    }};

    for entry in WalkDir::new(&pathbuf)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().unwrap_or(OsStr::new(""));

        if ext == OsStr::new("dll") || ext == OsStr::new("so") || ext == OsStr::new("dylib") {
            match unsafe { libloading::Library::new(path) } {
                Ok(lib) => {
                    let api = match unsafe { StackpackPluginAPI::from_library(&lib) } {
                        Ok(t) => t,
                        Err(e) => {
                            if_tracing! {{
                                tracing::error!(event = "plugins", path = ?path.display(), error = ?e, "plugin does not conform to Stackpack Plugin API");
                            }};
                            eprintln!("[WARN] plugin at {} does not conform to Stackpack Plugin API: {:?}", path.display(), e);
                            continue;
                        }
                    };
                    let plug = Plugin::new(path.to_path_buf(), api, lib);
                    let mut lock = LOADED_PLUGINS.lock();
                    lock.push(plug);
                    drop(lock);
                    if_tracing! {{
                        tracing::info!(event = "plugins", path = ?path.display(), "successfully loaded plugin");
                    }}
                }
                Err(e) => {
                    if_tracing! {{
                        tracing::error!(event = "plugins", path = ?path.display(), error = %e, "failed to load plugin");
                    }};
                    eprintln!("[WARN] failed to load plugin from {}: {}", path.display(), e);
                }
            }
        }
    }

    {
        let mut registry_lock = ALL_COMPRESSORS.lock();
        for (index, plug) in LOADED_PLUGINS.lock().iter().enumerate() {
            if_tracing! {{
                tracing::debug!(event = "registry", index = index, name = plug.api.short_name, path = ?plug.loaded_from.display(), "registered compressor");
            }};

            registry_lock.push(RegisteredCompressor::new_ffi(
                FfiMutator { plugin_index: index },
                plug.api.short_name,
                plug.api.description.as_option().copied(),
            ));
        }
    }
}

#[derive(Debug, Clone)]
pub struct FfiMutator {
    plugin_index: usize,
}

impl Mutator for FfiMutator {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        let api = &LOADED_PLUGINS.lock()[self.plugin_index].api;
        let mut ptr = buf.as_mut_ptr();
        let mut len = buf.len();
        let mut cap = buf.capacity();

        let result = unsafe { (api.drive_mutation)(data.as_ptr(), data.len(), &mut ptr, &mut len, &mut cap) };

        let mut new_vec = unsafe { Vec::from_raw_parts(ptr, len, cap) };
        mem::swap(&mut new_vec, buf);
        mem::forget(new_vec);

        if result.value {
            Ok(())
        } else {
            Err(anyhow::anyhow!("plugin drive mutation failed"))
        }
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        let api = &LOADED_PLUGINS.lock()[self.plugin_index].api;
        let mut ptr = buf.as_mut_ptr();
        let mut len = buf.len();
        let mut cap = buf.capacity();

        let result = unsafe { (api.revert_mutation)(data.as_ptr(), data.len(), &mut ptr, &mut len, &mut cap) };

        let mut new_vec = unsafe { Vec::from_raw_parts(ptr, len, cap) };
        mem::swap(&mut new_vec, buf);
        mem::forget(new_vec);

        if result.value {
            Ok(())
        } else {
            Err(anyhow::anyhow!("plugin revert mutation failed"))
        }
    }
}

pub unsafe fn unload_plugins() {
    let mut lock = LOADED_PLUGINS.lock();
    lock.clear();
}
