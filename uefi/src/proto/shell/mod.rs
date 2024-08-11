//! EFI Shell Protocol v2.2

use core::{ffi::c_void, mem::MaybeUninit};

use crate::proto::unsafe_protocol;
use crate::{CStr16, Event, Handle, Result, Status, StatusExt};
use uefi_raw::protocol::shell::*;

/// The Shell protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellProtocol::GUID)]
pub struct Shell(ShellProtocol);

impl Shell {
    /// TODO
    pub fn execute(
        &self,
        parent_image: Handle,
        command_line: &CStr16,
        environment: &[&CStr16],
    ) -> Result {
        let mut out_status: MaybeUninit<Status> = MaybeUninit::uninit();
        // This is required because the compiler won't turn `&[&Cstr16]`
        // or `*const &Cstr16 into *const *const CStr16`
        let environment = environment.as_ptr();
        let environment = environment.cast::<*const u16>();

        (self.0.execute)(
            parent_image.as_ptr().cast(),
            command_line.as_ptr().cast(),
            environment,
            out_status.as_mut_ptr(),
        )
        .to_result()
        //.to_result_with_val(|| unsafe { out_status.assume_init() })
    }

    /// Returns `true` if any script files are currently being processed.
    #[must_use]
    pub fn batch_is_active(&self) -> bool {
        (self.0.batch_is_active)()
    }

    /// Disables the page break output mode.
    pub fn disable_page_break(&self) {
        (self.0.disable_page_break)()
    }

    /// Enables the page break output mode.
    pub fn enable_page_break(&self) {
        (self.0.enable_page_break)()
    }

    /// Gets the file information from an open file handle
    // TODO: How do we free this automatically?
    // Doesn't seem to work!
    pub fn get_file_info(&self, file_handle: ShellFileHandle) -> Option<&FileInfo> {
        let info = (self.0.get_file_info)(file_handle);
        if info.is_null() {
            None
        } else {
            unsafe { info.as_ref() }
        }
    }

    /// Sets the file information to an opened file handle
    pub fn set_file_info(&self, file_handle: ShellFileHandle, file_info: &FileInfo) -> Result {
        (self.0.set_file_info)(file_handle, file_info).to_result()
    }

    /// Opens a file or a directory by file name
    pub fn open_file_by_name(
        &self,
        file_name: &[u16],
        mode: u64,
    ) -> Result<Option<ShellFileHandle>> {
        let mut out_file_handle: MaybeUninit<Option<ShellFileHandle>> = MaybeUninit::zeroed();
        (self.0.open_file_by_name)(
            file_name.as_ptr(),
            out_file_handle.as_mut_ptr().cast(),
            mode,
        )
        // Safety: if this call is successful, `out_file_handle`
        // will always be initialized and valid.
        .to_result_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    /// Closes `file_handle`. All data is flushed to the device and the file is closed.
    ///
    /// Per the UEFI spec, the file handle will be closed in all cases and this function
    /// only returns [`Status::SUCCESS`].
    pub fn close_file(&self, file_handle: ShellFileHandle) -> Result {
        (self.0.close_file)(file_handle).to_result()
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

        (self.0.create_file)(
            file_name.as_ptr().cast(),
            file_attribs,
            out_file_handle.as_mut_ptr().cast(),
        )
        // Safety: if this call is successful, `out_file_handle`
        // will always be initialized and valid.
        .to_result_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    /// Reads data from the file
    pub fn read_file(&self, file_handle: ShellFileHandle, buffer: &mut [u8]) -> Result {
        let mut read_size = buffer.len();
        (self.0.read_file)(
            file_handle,
            &mut read_size,
            buffer.as_mut_ptr() as *mut c_void,
        )
        .to_result()
    }

    /// Writes data to the file
    pub fn write_file(&self, file_handle: ShellFileHandle, buffer: &[u8]) -> Result {
        let mut read_size = buffer.len();
        (self.0.write_file)(
            file_handle,
            &mut read_size,
            buffer.as_ptr() as *const c_void,
        )
        .to_result()
    }

    /// TODO
    pub fn delete_file(&self, file_handle: ShellFileHandle) -> Result {
        (self.0.delete_file)(file_handle).to_result()
    }

    /// TODO
    pub fn delete_file_by_name(&self, file_name: &CStr16) -> Result {
        (self.0.delete_file_by_name)(file_name.as_ptr().cast()).to_result()
    }

    /// TODO
    pub fn find_files(&self, file_pattern: &CStr16) -> Result<Option<ShellFileIter>> {
        let mut out_list: MaybeUninit<*mut ShellFileInfo> = MaybeUninit::uninit();
        (self.0.find_files)(file_pattern.as_ptr().cast(), out_list.as_mut_ptr()).to_result_with_val(
            || {
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
            },
        )
    }

    /// Gets the size of a file
    pub fn get_file_size(&self, file_handle: ShellFileHandle) -> Result<u64> {
        let mut file_size: MaybeUninit<u64> = MaybeUninit::zeroed();
        (self.0.get_file_size)(file_handle, file_size.as_mut_ptr().cast())
            // Safety: if this call is successful, `out_file_handle`
            // will always be initialized and valid.
            .to_result_with_val(|| unsafe { file_size.assume_init() })
    }

    /// Major version of the shell
    pub fn major_version(&self) -> u32 {
        self.0.major_version
    }

    /// Minor version of the shell
    pub fn minor_version(&self) -> u32 {
        self.0.minor_version
    }

    /// Event to check if the user has requested execution to be stopped (CTRL-C)
    pub fn execution_break(&self) -> Option<Event> {
        unsafe { Event::from_ptr(self.0.execution_break) }
    }
}
