use std::mem;

use uuid::Uuid;

use symbolic_debuginfo::Object;
use symbolic_symcache::SymCache;

use core::{SymbolicStr, SymbolicUuid};
use debuginfo::SymbolicObject;

/// Represents a symbolic sym cache.
pub struct SymbolicSymCache;

/// Represents a single symbol after lookup.
#[repr(C)]
pub struct SymbolicSymbol {
    pub sym_addr: u64,
    pub instr_addr: u64,
    pub line: u32,
    pub symbol: SymbolicStr,
    pub filename: SymbolicStr,
    pub base_dir: SymbolicStr,
    pub comp_dir: SymbolicStr,
}

/// Represents a lookup result of one or more items.
#[repr(C)]
pub struct SymbolicLookupResult {
    pub items: *mut SymbolicSymbol,
    pub len: usize,
}


ffi_fn! {
    /// Creates a symcache from a given object.
    unsafe fn symbolic_symcache_from_object(sobj: *const SymbolicObject)
        -> Result<*mut SymbolicSymCache>
    {
        let cache = SymCache::from_object(&*(sobj as *const Object))?;
        Ok(Box::into_raw(Box::new(cache)) as *mut SymbolicSymCache)
    }
}

ffi_fn! {
    /// Frees a symcache object.
    unsafe fn symbolic_symcache_free(scache: *mut SymbolicSymCache) {
        if !scache.is_null() {
            let cache = scache as *mut SymCache<'static>;
            Box::from_raw(cache);
        }
    }
}

ffi_fn! {
    /// Returns the internal buffer of the symcache.
    ///
    /// The internal buffer is exactly `symbolic_symcache_get_size` bytes long.
    unsafe fn symbolic_symcache_get_bytes(scache: *const SymbolicSymCache) -> Result<*const u8> {
        let cache = scache as *mut SymCache<'static>;
        Ok((*cache).as_bytes().as_ptr())
    }
}

ffi_fn! {
    /// Returns the size in bytes of the symcache.
    unsafe fn symbolic_symcache_get_size(scache: *const SymbolicSymCache) -> Result<usize> {
        let cache = scache as *mut SymCache<'static>;
        Ok((*cache).size())
    }
}

ffi_fn! {
    /// Returns the architecture of the symcache.
    unsafe fn symbolic_symcache_get_arch(scache: *const SymbolicSymCache) -> Result<SymbolicStr> {
        let cache = scache as *mut SymCache<'static>;
        Ok(SymbolicStr::new((*cache).arch()?.name()))
    }
}

ffi_fn! {
    /// Returns the architecture of the symcache.
    unsafe fn symbolic_symcache_get_uuid(scache: *const SymbolicSymCache) -> Result<SymbolicUuid> {
        let cache = scache as *mut SymCache<'static>;
        Ok(mem::transmute(*(*cache).uuid().unwrap_or(Uuid::nil()).as_bytes()))
    }
}

ffi_fn! {
    /// Returns true if the symcache has line infos.
    unsafe fn symbolic_symcache_has_line_info(scache: *const SymbolicSymCache) -> Result<bool> {
        let cache = scache as *mut SymCache<'static>;
        Ok((*cache).has_line_info()?)
    }
}

ffi_fn! {
    /// Returns true if the symcache has file infos.
    unsafe fn symbolic_symcache_has_file_info(scache: *const SymbolicSymCache) -> Result<bool> {
        let cache = scache as *mut SymCache<'static>;
        Ok((*cache).has_file_info()?)
    }
}

ffi_fn! {
    /// Looks up a single symbol.
    unsafe fn symbolic_symcache_lookup(scache: *const SymbolicSymCache,
                                       addr: u64) -> Result<SymbolicLookupResult> {
        let cache = scache as *const SymCache<'static>;
        let vec = (*cache).lookup(addr)?;

        let mut items = vec![];
        for symbol in vec {
            items.push(SymbolicSymbol {
                sym_addr: symbol.sym_addr(),
                instr_addr: symbol.instr_addr(),
                line: symbol.line(),
                symbol: SymbolicStr::new(symbol.symbol()),
                filename: SymbolicStr::new(symbol.filename()),
                base_dir: SymbolicStr::new(symbol.base_dir()),
                comp_dir: SymbolicStr::new(symbol.comp_dir()),
            });
        }

        items.shrink_to_fit();
        let rv = SymbolicLookupResult {
            items: items.as_mut_ptr(),
            len: items.len(),
        };
        mem::forget(items);
        Ok(rv)
    }
}

ffi_fn! {
    /// Frees a lookup result.
    unsafe fn symbolic_lookup_result_free(slr: *mut SymbolicLookupResult) {
        if !slr.is_null() {
            Vec::from_raw_parts((*slr).items, (*slr).len, (*slr).len);
        }
    }
}
