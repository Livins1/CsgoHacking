mod hazedumper;
mod mem;
mod player;

use hazedumper::Haze;
use log::error;

use mem::{Module, Process};
use player::{EntityPlayer, Player};

use std::collections::btree_map::Iter;
use std::process::exit;

use std::thread::sleep;
use std::time::Duration;
use winapi::_core::marker::PhantomData;
use winapi::um::errhandlingapi::GetLastError;

pub struct CsgoRuntime {
    pub process: Process,
    pub client: usize,
    pub engine: usize,
    pub hazedumper: Haze,
}

impl CsgoRuntime {
    pub fn get_address(&self, offset: usize, client: bool) -> usize {
        if client {
            self.client + offset
        } else {
            self.engine + offset
        }
    }

    // pub unsafe fn get_local_player(&self) -> Option<usize> {

    // }

    #[inline]
    pub unsafe fn read_ptr<T>(&self, offset: usize, client: bool) -> Option<RemotePtr<T>> {
        let address: u32 = self
            .read_offset(offset, client)
            .expect(&*format!("Failed to read pointer: ox{:x}", offset));

        if address == 0 {
            None
        } else {
            Some(RemotePtr::<T> {
                address: address as usize,
                runtime: self,
                inner: PhantomData,
            })
        }
    }

    #[inline]
    pub unsafe fn read_offset<T>(&self, offset: usize, client: bool) -> Option<T> {
        let address = self.get_address(offset, client);
        self.process.read(address)
    }

    #[inline]
    pub unsafe fn write_offset<T>(&self, offset: usize, value: &T, client: bool) {
        let address = self.get_address(offset, client);
        self.process.write(address, value);
    }

    #[inline]
    pub unsafe fn get_entities(&self) -> impl Iterator<Item = EntityPlayer> {
        (1..64)
            .map(move |i| EntityPlayer::get(self, i))
            .flatten()
            .filter(|entity| {
                // entity.get_base_ptr().print_address();
                entity.is_alive()
                
            })
    }
}

#[derive(Clone)]
pub struct RemotePtr<'a, T> {
    address: usize,
    runtime: &'a CsgoRuntime,
    inner: PhantomData<T>,
}

impl<'a, T> RemotePtr<'a, T> {
    #[inline]
    pub unsafe fn read(&self) -> T {
        self.runtime
            .process
            .read(self.address)
            .expect(format!("Failed to read pointer: 0x{:16X}", self.address).as_str())
    }

    #[inline]
    pub unsafe fn write(&self, value: &T) -> bool {
        self.print_address();
        self.runtime.process.write(self.address, value)
    }

    #[inline]
    pub fn print_address(&self) {
        println!("Remote Ptr Address: 0x{:16x}", self.address);
    }

    #[inline]
    pub fn add(&self, offset: usize) -> Self {
        Self {
            address: self.address + offset,
            ..*self
        }
    }

    #[inline]
    pub fn cast<R>(&self) -> RemotePtr<R> {
        RemotePtr {
            address: self.address,
            runtime: self.runtime,
            inner: PhantomData,
        }
    }
}

fn init_csgo() -> CsgoRuntime {
    let exe_name = "csgo.exe";

    let process = mem::from_name(exe_name)
        .ok_or_else(|| {
            error!("Could not open process {}!", exe_name);
            exit(1);
        })
        .unwrap();

    let client = process
        .get_module("client.dll")
        .or_else(|| process.get_module("client_panorama.dll"))
        .unwrap();
    let engine = process.get_module("engine.dll").unwrap().base;

    let hazedumper = hazedumper::init_hazedumper();

    CsgoRuntime {
        process,
        client: client.base,
        engine,
        hazedumper,
    }
}

unsafe fn find_positions<'a>(eneitys: impl Iterator<Item = EntityPlayer<'a>>) {
    println!("Start FindEntities!");
    for entity in eneitys {
        // entity.get_team();
        entity.get_xy();

    }
}

fn main() {
    let csgo_runtime = init_csgo();
    println!("Runtime Init Success!");
    loop {
        unsafe {
            find_positions(csgo_runtime.get_entities());
        }

        sleep(Duration::from_millis(400));
    }
}
