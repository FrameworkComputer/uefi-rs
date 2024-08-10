use core::{ffi::c_void, ptr::NonNull};

use crate::{guid, Guid, Char16, Event, Handle, Status};
use crate::time::Time;
use crate::protocol::file_system::FileAttribute;

#[repr(u64)]
#[derive(Debug)]
/// Open modes
///
/// Can be added together to combine them.
pub enum FileOpenMode {
    /// Open file for reading
    Read = 0x0000000000000001,
    /// Open file for writing
    Write = 0x0000000000000002,
    /// Create the file if it doesn't exist already
    Create = 0x8000000000000000,
}

/// TODO
#[derive(Debug)]
#[repr(C)]
pub struct ShellProtocol {
    pub execute: extern "efiapi" fn(
        parent_image_handle: *const Handle,
        commandline: *const Char16,
        environment: *const *const Char16,
        out_status: *mut Status,
    ) -> Status,
    pub get_env: usize,
    pub set_env: usize,
    pub get_alias: usize,
    pub set_alias: usize,
    pub get_help_text: usize,
    pub get_device_path_from_map: usize,
    pub get_map_from_device_path: usize,
    pub get_device_path_from_file_path: usize,
    pub get_file_path_from_device_path: usize,
    pub set_map: usize,

    pub get_cur_dir: usize,
    pub set_cur_dir: usize,
    pub open_file_list: usize,
    pub free_file_list: usize,
    pub remove_dup_in_file_list: usize,

    pub batch_is_active: extern "efiapi" fn() -> bool,
    pub is_root_shell: usize,
    pub enable_page_break: extern "efiapi" fn(),
    pub disable_page_break: extern "efiapi" fn(),
    pub get_page_break: usize,
    pub get_device_name: usize,

    pub get_file_info: extern "efiapi" fn(file_handle: ShellFileHandle) -> *const FileInfo,
    pub set_file_info: extern "efiapi" fn(file_handle: ShellFileHandle, file_info: &FileInfo) -> Status,
    pub open_file_by_name: extern "efiapi" fn(
        path: *const u16,
        file_handle: *mut ShellFileHandle,
        open_mode: u64,
    ) -> Status,
    pub close_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    pub create_file: extern "efiapi" fn(
        file_name: *const Char16,
        file_attribs: u64,
        out_file_handle: *mut ShellFileHandle,
    ) -> Status,
    pub read_file: extern "efiapi" fn(
        file_handle: ShellFileHandle,
        read_size: &mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_file: extern "efiapi" fn(
        file_handle: ShellFileHandle,
        buffer: &mut usize,
        buffer: *const c_void,
    ) -> Status,
    pub delete_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    pub delete_file_by_name: extern "efiapi" fn(file_name: *const Char16) -> Status,
    pub get_file_position: usize,
    pub set_file_position: usize,
    pub flush_file: usize,
    pub find_files: extern "efiapi" fn(
        file_pattern: *const Char16,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    pub find_files_in_dir: usize,
    pub get_file_size: extern "efiapi" fn(file_handle: ShellFileHandle, size: *mut u64) -> Status,

    pub open_root: usize,
    pub open_root_by_handle: usize,

    /// Event to check if the user has requested execution to be stopped (CTRL-C)
    pub execution_break: Event,

    /// Major version of the shell
    pub major_version: u32,
    /// Minor version of the shell
    pub minor_version: u32,
    pub register_guid_name: usize,
    pub get_guid_name: usize,
    pub get_guid_from_name: usize,
    pub get_env_ex: usize,
}

impl ShellProtocol {
    pub const GUID: Guid = guid!("6302d008-7f9b-4f30-87ac-60c9fef5da4e");
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
    full_name: *const Char16,
    file_name: *const Char16,
    shell_file_handle: Handle,
    info: *mut FileInfo,
}

impl ShellFileInfo {
    pub fn from_list_entry_ptr(entry: *const ListEntry) -> Self {
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

// TODO: Already defined in uefi/src/proto/media/file/info.rs
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
pub struct FileInfo {
    size: u64,
    file_size: u64,
    physical_size: u64,
    create_time: Time,
    last_access_time: Time,
    modification_time: Time,
    attribute: FileAttribute,
    file_name: *const Char16,
}

/// TODO
#[derive(Debug)]
pub struct ShellFileIter {
    current_node: *const ListEntry,
}

impl ShellFileIter {
    pub fn from_file_info_ptr(ptr: *const ShellFileInfo) -> Self {
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
