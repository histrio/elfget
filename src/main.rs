use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::{Error, ErrorKind};
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::SeekFrom;

const NT_GNU_BUILD_ID:u32 = 3;
const NT_GO_BUILD_ID:u32 = 4;

fn main()  -> io::Result<()>{
    let v = env::args().collect::<Vec<String>>();
    if let [_, elf_filename, ] = &v[..] {
        let mut f = File::open(elf_filename)?;
        let ei_mag0 = f.read_u8()?;
        let ei_mag1 = f.read_u8()?;
        let ei_mag2 = f.read_u8()?;
        let ei_mag3 = f.read_u8()?;
        let ei_class = f.read_u8()?;
        let ei_data = f.read_u8()?;
        let ei_version = f.read_u8()?;
        let ei_osabi = f.read_u8()?;
        let ei_abiversion = f.read_u8()?;
        let ei_pad = f.read_u8()?;
        f.read_u8()?;
        f.read_u8()?;
        f.read_u8()?;
        f.read_u8()?;
        f.read_u8()?;
        f.read_u8()?;

        if (ei_mag0, ei_mag1, ei_mag2, ei_mag3) != (127, 69, 76, 70) {
            panic!("Not an ELF file.")
        }

        if ei_class != 2 {
            panic!("Not an ELF64 file.")
        }

        let e_type = f.read_u16::<LittleEndian>()?;
        let e_machine = f.read_u16::<LittleEndian>()?;
        let e_version = f.read_u32::<LittleEndian>()?;
        let e_entry = f.read_u64::<LittleEndian>()?;
        let e_phoff = f.read_u64::<LittleEndian>()?;
        let e_shoff = f.read_u64::<LittleEndian>()?;
        let e_flags = f.read_u32::<LittleEndian>()?;
        let e_ehsize = f.read_u16::<LittleEndian>()?;
        let e_phentsize = f.read_u16::<LittleEndian>()?;
        let e_phnum = f.read_u16::<LittleEndian>()?;
        let e_shentsize = f.read_u16::<LittleEndian>()?;
        let e_shnum = f.read_u16::<LittleEndian>()?;
        let e_shstrndx = f.read_u16::<LittleEndian>()?;

        if e_phoff == 0 {
            panic!("Program headers not found.")
        }

        f.seek(SeekFrom::Start(e_phoff))?;
        for idx in 0..e_phnum {
            let p_type = f.read_u32::<LittleEndian>()?;
            let p_flags = f.read_u32::<LittleEndian>()?;
            let p_offset = f.read_u64::<LittleEndian>()?;
            let p_vaddr = f.read_u64::<LittleEndian>()?;
            let p_paddr = f.read_u64::<LittleEndian>()?;
            let p_filesz = f.read_u64::<LittleEndian>()?;
            let p_memsz = f.read_u64::<LittleEndian>()?;
            let p_align= f.read_u64::<LittleEndian>()?;

            if p_type == 4 {
                let p_end = p_offset + p_filesz;
                f.seek(SeekFrom::Start(e_phoff))?;

                let mut n_type = 0;
                let mut pos = f.seek(SeekFrom::Current(0))?;

                while n_type != NT_GNU_BUILD_ID && n_type != NT_GO_BUILD_ID && pos<=p_end {

                    let mut n_namesz = f.read_u32::<LittleEndian>()?;
                    let mut n_descsz = f.read_u32::<LittleEndian>()?;
                    n_type = f.read_u32::<LittleEndian>()?;

                    if (n_namesz % 4) != 0 {
                        n_namesz = ((n_namesz / 4) + 1) * 4;
                    }

                    if (n_descsz % 4) != 0 {
                        n_descsz = ((n_descsz / 4) + 1) * 4;
                    }

                    let mut name = vec![0u8; n_namesz as usize];
                    f.read(name.as_mut_slice())?;

                    let mut desc = vec![0u8; n_descsz as usize];
                    f.read(desc.as_mut_slice())?;

                    pos = f.seek(SeekFrom::Current(0))?;
                }
                if n_type != 0 {
                    println!("{} = {} = {}", n_type, pos, p_end);
                    return Ok(())
                }
            }
        }
        panic!("Program header PT_NOTE with NT_GNU_BUILD_ID was not found.")

    } else {
        panic!("wrong parameter");
    }
}
