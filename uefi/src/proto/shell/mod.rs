//! EFI Shell Protocol v2.2

use core::{ffi::c_void, mem::MaybeUninit, ptr::NonNull};

use crate::proto::unsafe_protocol;
use crate::{CStr16, Event, Handle, Result, Status};

use super::media::file::FileInfo;

#[repr(u64)]
#[derive(Debug)]
pub enum FileOpenMode {
    Read = 0x0000000000000001,
    Write = 0x0000000000000002,
    Create = 0x8000000000000000,
}

/// TODO
#[repr(C)]
#[unsafe_protocol("6302d008-7f9b-4f30-87ac-60c9fef5da4e")]
pub struct Shell {
    execute: extern "efiapi" fn(
        parent_image_handle: *const Handle,
        commandline: *const CStr16,
        environment: *const *const CStr16,
        out_status: *mut Status,
    ) -> Status,
    get_env: usize,
    set_env: usize,
    get_alias: usize,
    set_alias: usize,
    get_help_text: usize,
    get_device_path_from_map: usize,
    get_map_from_device_path: usize,
    get_device_path_from_file_path: usize,
    get_file_path_from_device_path: usize,
    set_map: usize,

    get_cur_dir: usize,
    set_cur_dir: usize,
    open_file_list: usize,
    free_file_list: usize,
    remove_dup_in_file_list: usize,

    batch_is_active: extern "efiapi" fn() -> bool,
    is_root_shell: usize,
    enable_page_break: extern "efiapi" fn(),
    disable_page_break: extern "efiapi" fn(),
    get_page_break: usize,
    get_device_name: usize,

    get_file_info: extern "efiapi" fn(file_handle: ShellFileHandle) -> *const FileInfo,
    set_file_info: extern "efiapi" fn(file_handle: ShellFileHandle, file_info: &FileInfo) -> Status,
    open_file_by_name: extern "efiapi" fn(
        path: *const u16,
        file_handle: *mut ShellFileHandle,
        open_mode: u64,
    ) -> Status,
    close_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    create_file: extern "efiapi" fn(
        file_name: &CStr16,
        file_attribs: u64,
        out_file_handle: *mut ShellFileHandle,
    ) -> Status,
    read_file: extern "efiapi" fn(
        file_handle: ShellFileHandle,
        read_size: &mut usize,
        buffer: *mut c_void,
    ) -> Status,
    write_file: extern "efiapi" fn(
        file_handle: ShellFileHandle,
        buffer: &mut usize,
        buffer: *const c_void,
    ) -> Status,
    delete_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    delete_file_by_name: extern "efiapi" fn(file_name: &CStr16) -> Status,
    get_file_position: usize,
    set_file_position: usize,
    flush_file: usize,
    find_files: extern "efiapi" fn(
        file_pattern: *const CStr16,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    find_files_in_dir: usize,
    get_file_size: extern "efiapi" fn(file_handle: ShellFileHandle, size: *mut u64) -> Status,

    open_root: usize,
    open_root_by_handle: usize,

    pub execution_break: Event,

    pub major_version: u32,
    pub minor_version: u32,
    register_guid_name: usize,
    get_guid_name: usize,
    get_guid_from_name: usize,
    get_env_ex: usize,
}

impl Shell {
    /// TODO
    pub fn execute(
        &self,
        parent_image: Handle,
        command_line: &CStr16,
        environment: &[&CStr16],
    ) -> Result<Status> {
        // This is required because the compiler won't turn `&[&Cstr16]`
        // or `*const &Cstr16 into *const *const CStr16`
        let mut out_status: MaybeUninit<Status> = MaybeUninit::uninit();
        let environment = environment.as_ptr();
        let environment = environment.cast::<*const CStr16>();

        (self.execute)(
            &parent_image,
            command_line,
            environment,
            out_status.as_mut_ptr(),
        )
        .into_with_val(|| unsafe { out_status.assume_init() })
    }

    /// Returns `true` if any script files are currently being processed.
    #[must_use]
    pub fn batch_is_active(&self) -> bool {
        (self.batch_is_active)()
    }

    /// Disables the page break output mode.
    pub fn disable_page_break(&self) {
        (self.disable_page_break)()
    }

    /// Enables the page break output mode.
    pub fn enable_page_break(&self) {
        (self.enable_page_break)()
    }

    // TODO: How do we free this automatically?
    pub fn get_file_info(&self, file_handle: ShellFileHandle) -> Option<&FileInfo> {
        let info = (self.get_file_info)(file_handle);
        if info.is_null() {
            None
        } else {
            unsafe { info.as_ref() }
        }
    }

    pub fn set_file_info(&self, file_handle: ShellFileHandle, file_info: &FileInfo) -> Result<()> {
        (self.set_file_info)(file_handle, file_info).into()
    }

    pub fn open_file_by_name(
        &self,
        file_name: &[u16],
        mode: u64,
    ) -> Result<Option<ShellFileHandle>> {
        let mut out_file_handle: MaybeUninit<Option<ShellFileHandle>> = MaybeUninit::zeroed();
        (self.open_file_by_name)(
            file_name.as_ptr(),
            out_file_handle.as_mut_ptr().cast(),
            mode,
        )
        // Safety: if this call is successful, `out_file_handle`
        // will always be initialized and valid.
        .into_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    /// Closes `file_handle`. All data is flushed to the device and the file is closed.
    ///
    /// Per the UEFI spec, the file handle will be closed in all cases and this function
    /// only returns [`Status::SUCCESS`].
    pub fn close_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.close_file)(file_handle).into()
    }

    /// TODO
    pub fn create_file(
        &self,
        file_name: &CStr16,
        file_attribs: u64,
    ) -> Result<Option<ShellFileHandle>> {
        // TODO: Find out how we could take a &str instead, or maybe AsRef<str>, though I think it needs `alloc`
        // the returned handle can possibly be NULL, so we need to wrap `ShellFileHandle` in an `Option`
        let mut out_file_handle: MaybeUninit<Option<ShellFileHandle>> = MaybeUninit::zeroed();

        (self.create_file)(file_name, file_attribs, out_file_handle.as_mut_ptr().cast())
            // Safety: if this call is successful, `out_file_handle`
            // will always be initialized and valid.
            .into_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    pub fn read_file(&self, file_handle: ShellFileHandle, buffer: &mut [u8]) -> Result<()> {
        let mut read_size = buffer.len();
        (self.read_file)(
            file_handle,
            &mut read_size,
            buffer.as_mut_ptr() as *mut c_void,
        )
        .into()
    }

    pub fn write_file(&self, file_handle: ShellFileHandle, buffer: &[u8]) -> Result<()> {
        let mut read_size = buffer.len();
        (self.write_file)(
            file_handle,
            &mut read_size,
            buffer.as_ptr() as *const c_void,
        )
        .into()
    }

    /// TODO
    pub fn delete_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.delete_file)(file_handle).into()
    }

    /// TODO
    pub fn delete_file_by_name(&self, file_name: &CStr16) -> Result<()> {
        (self.delete_file_by_name)(file_name).into()
    }

    /// TODO
    pub fn find_files(&self, file_pattern: &CStr16) -> Result<Option<ShellFileIter>> {
        let mut out_list: MaybeUninit<*mut ShellFileInfo> = MaybeUninit::uninit();
        (self.find_files)(file_pattern, out_list.as_mut_ptr()).into_with_val(|| {
            unsafe {
                // safety: this is initialized after this call succeeds, even if it's
                // null
                let out_list = out_list.assume_init();
                if out_list.is_null() {
                    // no files found
                    None
                } else {
                    Some(ShellFileIter::from_file_info_ptr(out_list))
                }
            }
        })
    }

    pub fn get_file_size(&self, file_handle: ShellFileHandle) -> Result<u64> {
        let mut file_size: MaybeUninit<u64> = MaybeUninit::zeroed();
        (self.get_file_size)(file_handle, file_size.as_mut_ptr().cast())
            // Safety: if this call is successful, `out_file_handle`
            // will always be initialized and valid.
            .into_with_val(|| unsafe { file_size.assume_init() })
    }
}

/// TODO
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ShellFileHandle(NonNull<c_void>);

/// TODO
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShellFileInfo {
    link: ListEntry,
    status: Status,
    full_name: *const CStr16,
    file_name: *const CStr16,
    shell_file_handle: Handle,
    info: *mut FileInfo,
}

impl ShellFileInfo {
    fn from_list_entry_ptr(entry: *const ListEntry) -> Self {
        // Safety: This is safe due to the C representation of the two structs;
        // Every [`ShellFileInfo`] starts with a [`ListEntry`] so a pointer to [`ListEntry`]
        // is a pointer to [`ShellFileInfo`] as well.
        unsafe { *entry.cast::<ShellFileInfo>() }
    }
}

/// TODO
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListEntry {
    flink: *mut ListEntry,
    blink: *mut ListEntry,
}

/// TODO
#[derive(Debug)]
pub struct ShellFileIter {
    current_node: *const ListEntry,
}

impl ShellFileIter {
    fn from_file_info_ptr(ptr: *const ShellFileInfo) -> Self {
        // Safety: This is safe, as each [`ShellFileInfo`] begins
        // with a [`ListEntry`] and are #[repr(C)] structs
        Self {
            current_node: ptr.cast::<ListEntry>(),
        }
    }
}

impl Iterator for ShellFileIter {
    type Item = ShellFileInfo;

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: This is safe as we're dereferencing a pointer that we've already null-checked
        unsafe {
            if (*self.current_node).flink.is_null() {
                None
            } else {
                let ret = ShellFileInfo::from_list_entry_ptr((*self.current_node).flink);
                self.current_node = (*self.current_node).flink;
                Some(ret)
            }
        }
    }
}
