use std::ffi::c_void;
use std::mem::size_of_val;
use std::{fmt::Debug, mem::size_of, ptr::null_mut};

pub mod error;
use error::{ReadWriteMemoryFailedDetail, SnapshotFailedDetail, TAExternalError};

use winapi::ctypes::c_void as winapi_cvoid;
use winapi::shared::minwindef::DWORD;
use winapi::um::memoryapi::{VirtualProtectEx, VirtualQueryEx};
use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, PAGE_READWRITE};
use winapi::{
    shared::{
        basetsd::SIZE_T,
        minwindef::{FALSE, HMODULE, LPCVOID, LPVOID, TRUE},
    },
    um::{
        errhandlingapi::GetLastError,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        processthreadsapi::OpenProcess,
        tlhelp32::{
            CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next, MODULEENTRY32,
            PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
        },
        winnt::{HANDLE, PROCESS_ALL_ACCESS},
    },
};

#[derive(Debug)]
pub struct Module {
    process_handle: HANDLE,
    pub module_size: u32,
    pub module_base_address: usize,
    pub module_handle: HMODULE,
    pub module_name: String,
    pub module_path: String,
}

impl Default for Module {
    fn default() -> Self {
        Module {
            process_handle: 0x0 as HANDLE,
            module_size: 0,
            module_base_address: 0,
            module_handle: 0x0 as HMODULE,
            module_name: String::default(),
            module_path: String::default(),
        }
    }
}

impl Module {
    fn from_module_entry(process_handle: HANDLE, module_entry: &MODULEENTRY32, module_name: String) -> Self {
        Module {
            process_handle,
            module_size: module_entry.modBaseSize,
            module_base_address: module_entry.modBaseAddr as usize,
            module_handle: module_entry.hModule,
            module_name,
            // This is allowed because szExePath.as_ptr() is the address within module_entry variable, not the address in the target process.
            module_path: unsafe { read_null_terminated_string(module_entry.szExePath.as_ptr() as usize) }.unwrap(),
        }
    }
}

/// read fetches the value that given address is holding.
/// * `base_address` - the address that is supposed to have the value you want
pub fn read<T>(process_handle: HANDLE, base_address: usize) -> Result<T, TAExternalError> {
    unsafe {
        let mut memory_info: MEMORY_BASIC_INFORMATION = MEMORY_BASIC_INFORMATION::default();
        VirtualQueryEx(
            process_handle,
            base_address as LPCVOID,
            &mut memory_info,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        let is_readable = is_page_readable(&memory_info);
        let mut old_protect = PAGE_READWRITE;
        let mut new_protect = PAGE_READWRITE;
        if !is_readable {
            VirtualProtectEx(
                process_handle,
                base_address as LPVOID,
                size_of::<LPVOID>(),
                new_protect,
                &mut old_protect as *mut DWORD,
            );
        }
        let mut buffer: T = std::mem::zeroed::<T>();
        let ok = ReadProcessMemory(
            process_handle,
            base_address as LPCVOID,
            &mut buffer as *mut _ as LPVOID,
            size_of_val(&buffer) as SIZE_T,
            null_mut::<SIZE_T>(),
        );
        if !is_readable {
            VirtualProtectEx(
                process_handle,
                base_address as LPVOID,
                size_of::<LPVOID>(),
                old_protect,
                &mut new_protect as *mut DWORD,
            );
        }
        if ok == FALSE {
            let error_code = GetLastError();
            return match error_code {
                6 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidHandle,
                )),
                299 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorPartialCopy,
                )),
                487 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidAddress,
                )),
                _ => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::UnknownError { error_code },
                )),
            };
        }
        Ok(buffer)
    }
}

pub fn read_buffer(process_handle: HANDLE, base_address: usize, size: usize) -> Result<Vec<u8>, TAExternalError> {
    unsafe {
        let mut memory_info: MEMORY_BASIC_INFORMATION = MEMORY_BASIC_INFORMATION::default();
        VirtualQueryEx(
            process_handle,
            base_address as LPCVOID,
            &mut memory_info,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        let is_readable = is_page_readable(&memory_info);
        let mut old_protect = PAGE_READWRITE;
        let mut new_protect = PAGE_READWRITE;
        if !is_readable {
            VirtualProtectEx(
                process_handle,
                base_address as LPVOID,
                size as SIZE_T,
                new_protect,
                &mut old_protect as *mut DWORD,
            );
        }
        let mut buffer: Vec<u8> = vec![0; size];
        let ok = ReadProcessMemory(
            process_handle,
            base_address as LPCVOID,
            buffer.as_mut_ptr() as *mut _ as LPVOID,
            size as SIZE_T,
            null_mut::<SIZE_T>(),
        );
        if !is_readable {
            VirtualProtectEx(
                process_handle,
                base_address as LPVOID,
                size_of::<LPVOID>(),
                old_protect,
                &mut new_protect as *mut DWORD,
            );
        }
        if ok == FALSE {
            let error_code = GetLastError();
            return match error_code {
                6 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidHandle,
                )),
                299 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorPartialCopy,
                )),
                487 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidAddress,
                )),
                _ => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::UnknownError { error_code },
                )),
            };
        }
        Ok(buffer)
    }
}

/// write overwrites the value that given base_address is holding.
/// * `base_address` - the address that is supposed have the value you want to tamper with.
/// * `value` - new value you wanna overwrite
pub fn write<T>(process_handle: HANDLE, base_address: usize, value: &mut T) -> Result<(), TAExternalError> {
    unsafe {
        let ok = WriteProcessMemory(
            process_handle,
            base_address as LPVOID,
            value as *mut T as LPCVOID,
            size_of::<T>() as SIZE_T,
            null_mut::<SIZE_T>(),
        );
        if ok == FALSE {
            let error_code = GetLastError();
            return match error_code {
                6 => Err(TAExternalError::ReadMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidHandle,
                )),
                299 => Err(TAExternalError::WriteMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorPartialCopy,
                )),
                487 => Err(TAExternalError::WriteMemoryFailed(
                    ReadWriteMemoryFailedDetail::ErrorInvalidAddress,
                )),
                _ => Err(TAExternalError::WriteMemoryFailed(
                    ReadWriteMemoryFailedDetail::UnknownError { error_code },
                )),
            };
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Process<'a> {
    pub process_name: &'a str,
    pub process_id: u32,
    pub process_handle: HANDLE,
}

impl<'a> Default for Process<'a> {
    fn default() -> Self {
        Process {
            process_name: "",
            process_id: 0,
            process_handle: 0x0 as HANDLE,
        }
    }
}

impl<'a> Process<'a> {
    pub fn from_process_name(process_name: &'a str) -> Result<Self, TAExternalError> {
        let process_id = get_process_id(process_name)?;
        let process_handle = get_process_handle(process_id);
        Ok(Process {
            process_name,
            process_id,
            process_handle,
        })
    }

    pub fn get_module_info(&self, module_name: &str) -> Result<Module, TAExternalError> {
        unsafe {
            let snap_handle = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, self.process_id);
            if snap_handle == INVALID_HANDLE_VALUE {
                return Err(TAExternalError::SnapshotFailed(SnapshotFailedDetail::InvalidHandle));
            }
            let mut module_entry: MODULEENTRY32 = MODULEENTRY32::default();
            module_entry.dwSize = size_of::<MODULEENTRY32>() as u32;
            if Module32First(snap_handle, &mut module_entry) == TRUE {
                if read_null_terminated_string(module_entry.szModule.as_ptr() as usize).unwrap() == module_name {
                    return Ok(Module::from_module_entry(
                        self.process_handle,
                        &module_entry,
                        module_name.into(),
                    ));
                }
                loop {
                    if Module32Next(snap_handle, &mut module_entry) == FALSE {
                        if GetLastError() == 18 {
                            CloseHandle(snap_handle);
                            return Err(TAExternalError::SnapshotFailed(SnapshotFailedDetail::NoMoreFiles));
                        }
                    }
                    if read_null_terminated_string(module_entry.szModule.as_ptr() as usize).unwrap() == module_name {
                        CloseHandle(snap_handle);
                        return Ok(Module::from_module_entry(
                            self.process_handle,
                            &module_entry,
                            module_name.into(),
                        ));
                    }
                }
            }
            CloseHandle(snap_handle);
            Err(TAExternalError::ModuleNotFound)
        }
    }

    pub fn get_module_base(&self, module_name: &str) -> Result<usize, TAExternalError> {
        let info: Module = self.get_module_info(module_name)?;
        Ok(info.module_base_address)
    }
}
pub fn close_proc_handle(process_handle: *mut winapi_cvoid) {
    unsafe {
        let x = CloseHandle(process_handle);
        println!("{:?}", x);
    }
}
fn get_process_id(process_name: &str) -> Result<u32, TAExternalError> {
    unsafe {
        let snap_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap_handle == INVALID_HANDLE_VALUE {
            return Err(TAExternalError::SnapshotFailed(SnapshotFailedDetail::InvalidHandle));
        }
        let mut proc_entry: PROCESSENTRY32 = PROCESSENTRY32::default();
        proc_entry.dwSize = size_of::<PROCESSENTRY32>() as u32;
        if Process32First(snap_handle, &mut proc_entry) == 1 {
            if read_null_terminated_string(proc_entry.szExeFile.as_ptr() as usize).unwrap() == process_name {
                return Ok(proc_entry.th32ProcessID as u32);
            }
            loop {
                if Process32Next(snap_handle, &mut proc_entry) == FALSE {
                    if GetLastError() == 18 {
                        return Err(TAExternalError::SnapshotFailed(SnapshotFailedDetail::NoMoreFiles));
                    }
                }
                if read_null_terminated_string(proc_entry.szExeFile.as_ptr() as usize).unwrap() == process_name {
                    return Ok(proc_entry.th32ProcessID as u32);
                }
            }
        }
        CloseHandle(snap_handle);
    }
    Err(TAExternalError::ProcessNotFound)
}

use winapi::um::winnt::{MEM_COMMIT, PAGE_NOACCESS};

pub fn is_page_readable(memory_info: &MEMORY_BASIC_INFORMATION) -> bool {
    if memory_info.State != MEM_COMMIT || memory_info.Protect == 0x0 || memory_info.Protect == PAGE_NOACCESS {
        return false;
    }
    true
}

use std::str::Utf8Error;

pub(crate) unsafe fn read_null_terminated_string(base_address: usize) -> Result<String, Utf8Error> {
    let mut name: Vec<u8> = Vec::new();
    let mut i: isize = 0;
    loop {
        let char_as_u8 = *(base_address as *const u8).offset(i);
        if char_as_u8 == 0 {
            return Ok(std::str::from_utf8(&name[..])?.to_owned());
        }
        name.push(char_as_u8);
        i += 1;
    }
}

fn get_process_handle(process_id: u32) -> HANDLE {
    unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, process_id as u32) }
}

#[test]
#[ignore]
fn test_get_process_id() {
    let process_name = "csgo.exe";
    assert_ne!(0, get_process_id(process_name).unwrap());
}

#[test]
#[ignore]
fn test_get_process_handle() {
    let process_name = "csgo.exe";
    let process_id = get_process_id(process_name).unwrap();
    assert_ne!(0x0, get_process_handle(process_id) as i32);
}
