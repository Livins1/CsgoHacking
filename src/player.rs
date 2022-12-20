use crate::{CsgoRuntime as Runtime, RemotePtr};
use std::fmt::DebugSet;
use std::thread::sleep;
use std::time::Duration;
use std::{mem, ptr};
use winapi::_core::marker::PhantomData;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPCVOID, LPVOID, PBOOL, PDWORD};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, WriteProcessMemory};

/// Entity instance
use winapi::um::winnt::PAGE_READWRITE;
pub struct EntityPlayer<'a> {
    /// The runtime reference
    runtime: &'a Runtime,
    /// the base entity pointer
    inner: RemotePtr<'a, usize>,
}

unsafe impl<'a> Player<'a> for EntityPlayer<'a> {
    /// Returns the base entity player pointer (client.dll module offset + dwEntityList signature offset * (entity index * 0x10))
    fn get_base_ptr(&self) -> RemotePtr<'a, usize> {
        self.inner.clone()
    }

    /// Returns the runtime reference
    fn get_runtime(&self) -> &'a Runtime {
        self.runtime
    }
}

impl<'a> EntityPlayer<'a> {
    /// Returns entity player instance by index
    ///
    /// # Arguments
    ///
    /// * `runtime` - The runtime reference to read the signature
    /// * `index` The entity index to read
    pub unsafe fn get(runtime: &'a Runtime, index: i32) -> Option<EntityPlayer<'a>> {
        if index <= 0 {
            return None;
        }

        let inner = runtime.read_ptr::<usize>(
            runtime.hazedumper.signatures.dwEntityList.unwrap() as usize + (index as usize * 0x10),
            true,
        )?;

        Some(EntityPlayer { runtime, inner })
        // let offset =
        //     runtime.hazedumper.signatures.dwEntityList.unwrap() as usize + (index as usize * 0x10);

        // let address: u32 = runtime
        //     .read_offset(offset, true)
        //     .expect(&*format!("Failed to read pointer: ox{:x}", offset));

        // let inner = RemotePtr {
        //     address: address as usize,
        //     runtime,
        //     inner: PhantomData,
        // };

        // Some(EntityPlayer { runtime, inner })
    }
}

pub unsafe trait Player<'a> {
    fn get_base_ptr(&self) -> RemotePtr<'a, usize>;
    fn get_runtime(&self) -> &'a Runtime;

    /// Returns player alive state
    #[inline]
    unsafe fn is_alive(&self) -> bool {
        let health = self.get_health();
        (0..=100).contains(&health)
    }

    #[inline]
    unsafe fn get_xy(&self) {
        let ent_addr = self.get_base_ptr().address;

        let origin_address =
            ent_addr + self.get_runtime().hazedumper.netvars.m_vecOrigin.unwrap() as usize;

        let z_address = origin_address + 8;
        let y_address = origin_address + 4;

        let x: f32 = self.get_runtime().process.read(origin_address).unwrap();
        let y: f32 = self.get_runtime().process.read(y_address).unwrap();
        println!("x:{}, y:{}", x, y);
    }

    #[inline]
    unsafe fn set_spotted(&self) {
        // self.get_base_ptr()
        //     .add(self.get_runtime().hazedumper.netvars.m_bSpotted.unwrap() as usize)
        //     .cast()
        //     .write(&1);
        self.get_base_ptr().print_address();
        let address = self.get_base_ptr().address
            + self.get_runtime().hazedumper.netvars.m_bSpotted.unwrap() as usize;

        let var = 2;
        if !self.get_runtime().process.write(address, &var) {
            println!("Write Error");
            println!("GetLastError: {}", GetLastError())
        };
    }

    /// Returns player health
    #[inline]
    unsafe fn get_health(&self) -> usize {
        let health: u32 = self
            .get_base_ptr()
            .add(self.get_runtime().hazedumper.netvars.m_iHealth.unwrap() as usize)
            .cast()
            .read();

        // println!("get_health value: {}", health);

        health as usize

        // let address = self
        //     .get_base_ptr()
        //     .add(self.get_runtime().hazedumper.netvars.m_iHealth.unwrap() as usize + 20000)
        //     .address;

        // let mut buffer = unsafe { mem::zeroed::<T>() };
        // println!("Process ReadAddress GetHealth: {:16X}", address);

        // let mut old_protect: DWORD = PAGE_READWRITE;
        // const N_SIZE: usize = 256;
        // let mut chunk: Vec<u8> = vec![0; N_SIZE];

        // match unsafe {
        //     // VirtualProtectEx(
        //     //     self.get_runtime().process.handle,
        //     //     address as LPVOID,
        //     //     2560,
        //     //     PAGE_READWRITE,
        //     //     &mut old_protect,
        //     // );
        //     ReadProcessMemory(
        //         self.get_runtime().process.handle,
        //         address as LPCVOID,
        //         &mut buffer as *mut T as LPVOID,
        //         mem::size_of::<T>() as SIZE_T,
        //         ptr::null_mut::<SIZE_T>(),
        //     )

        //     // ReadProcessMemory(k
        //     //     self.get_runtime().process.handle,
        //     //     address as LPVOID,
        //     //     // &mut buffer as *mut usize as LPVOID,
        //     //     &chunk.as_mut_ptr() as LPVOID,
        //     //     N_SIZE as SIZE_T,
        //     //     ptr::null_mut(),
        //     // )
        // } {
        //     FALSE => {
        //         unsafe {
        //             let err = GetLastError();
        //             println!("ReadProcessMemory GetLastError: {:?}", err)
        //         }
        //         0
        //     }
        //     _ => {
        //         println!("GetSomeThing:{:?} ", buffer);
        //         buffer as i8
        //     }
        // }
    }

    /// Returns player team value
    #[inline]
    unsafe fn get_team(&self) -> usize {
        let team: i32 = self
            .get_base_ptr()
            .add(self.get_runtime().hazedumper.netvars.m_iTeamNum.unwrap() as usize)
            .cast()
            .read();
        println!("TeamNum: {}", team);
        team as usize
    }

    // only local player
    #[inline]
    unsafe fn force_jump(&self) {
        self.get_runtime().write_offset(
            self.get_runtime()
                .hazedumper
                .signatures
                .dwForceJump
                .unwrap() as usize,
            &5,
            true,
        );
        sleep(Duration::from_millis(1));
        self.get_runtime().write_offset(
            self.get_runtime()
                .hazedumper
                .signatures
                .dwForceJump
                .unwrap() as usize,
            &4,
            true,
        );
    }
}

pub struct XYPosition {
    x: f32,
    y: f32,
}

impl XYPosition {
    pub fn print(&self) {
        println!("X: {}, Y: {}", self.x, self.y);
    }
}
